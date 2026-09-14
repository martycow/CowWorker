use crate::Store;
use rusqlite::{Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    path::{Component, Path},
};

type Result<T> = std::result::Result<T, String>;
fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Manifest {
    pub format_version: u32,
    pub schema_version: i64,
    pub created_at: String,
    pub files: BTreeMap<String, String>,
    pub counts: BTreeMap<String, i64>,
}

pub(crate) fn hash(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path).map_err(err)?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 65536];
    loop {
        let n = file.read(&mut buffer).map_err(err)?;
        if n == 0 {
            break;
        }
        digest.update(&buffer[..n]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn references(db: &Connection) -> Result<Vec<String>> {
    let mut stmt = db
        .prepare("SELECT id,file_name FROM document_versions")
        .map_err(err)?;
    let rows = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
        .map_err(err)?;
    let mut paths = vec![];
    for row in rows {
        let (id, name) = row.map_err(err)?;
        if uuid::Uuid::parse_str(&id).is_err() || name != format!("{id}.txt") {
            return Err("Invalid document asset reference.".into());
        }
        paths.push(format!("documents/{name}"));
    }
    let has_sources: bool = db
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name='sources' AND type='table')",
            [],
            |r| r.get(0),
        )
        .map_err(err)?;
    if has_sources {
        let mut stmt = db
            .prepare("SELECT id,file_name FROM sources")
            .map_err(err)?;
        for row in stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
            .map_err(err)?
        {
            let (id, name) = row.map_err(err)?;
            if uuid::Uuid::parse_str(&id).is_err() || name != format!("{id}.bin") {
                return Err("Invalid source asset reference.".into());
            }
            paths.push(format!("sources/{name}"));
        }
    }
    Ok(paths)
}

fn counts(db: &Connection) -> Result<BTreeMap<String, i64>> {
    let mut stmt = db.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name").map_err(err)?;
    let tables = stmt
        .query_map([], |r| r.get::<_, String>(0))
        .map_err(err)?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(err)?;
    tables
        .into_iter()
        .map(|table| {
            let count = db
                .query_row(
                    &format!("SELECT COUNT(*) FROM \"{}\"", table.replace('"', "\"\"")),
                    [],
                    |r| r.get(0),
                )
                .map_err(err)?;
            Ok((table, count))
        })
        .collect()
}

fn integrity(db: &Connection) -> Result<()> {
    let check: String = db
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))
        .map_err(err)?;
    let broken: i64 = db
        .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |r| {
            r.get(0)
        })
        .map_err(err)?;
    if check != "ok" || broken != 0 {
        return Err("Workspace integrity check failed. Restore a verified backup.".into());
    }
    Ok(())
}

pub(crate) fn create(db: &Connection, workspace: &Path, destination: &Path) -> Result<Manifest> {
    if destination.exists() {
        return Err("Choose a new backup directory. Existing files will not be replaced.".into());
    }
    fs::create_dir_all(destination).map_err(err)?;
    // VACUUM INTO includes committed WAL data in a consistent standalone snapshot.
    db.execute(
        "VACUUM INTO ?1",
        [destination
            .join("cowworker.db")
            .to_str()
            .ok_or("Invalid backup path")?],
    )
    .map_err(err)?;
    let snapshot = Connection::open_with_flags(
        destination.join("cowworker.db"),
        OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .map_err(err)?;
    integrity(&snapshot)?;
    let mut files = BTreeMap::new();
    for relative in references(&snapshot)? {
        let target = destination.join(&relative);
        fs::create_dir_all(target.parent().ok_or("Invalid asset path")?).map_err(err)?;
        fs::copy(workspace.join(&relative), &target)
            .map_err(|e| format!("Backup asset {relative}: {e}"))?;
        fs::OpenOptions::new()
            .write(true)
            .open(&target)
            .map_err(err)?
            .sync_all()
            .map_err(err)?;
        files.insert(relative, hash(&target)?);
    }
    files.insert(
        "cowworker.db".into(),
        hash(&destination.join("cowworker.db"))?,
    );
    let manifest = Manifest {
        format_version: 1,
        schema_version: snapshot
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .map_err(err)?,
        created_at: chrono::Utc::now().to_rfc3339(),
        files,
        counts: counts(&snapshot)?,
    };
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination.join("manifest.json"))
        .map_err(err)?;
    file.write_all(&serde_json::to_vec_pretty(&manifest).map_err(err)?)
        .map_err(err)?;
    file.sync_all().map_err(err)?;
    Ok(manifest)
}

pub fn verify(directory: &Path) -> Result<Manifest> {
    let manifest: Manifest =
        serde_json::from_slice(&fs::read(directory.join("manifest.json")).map_err(err)?)
            .map_err(err)?;
    if manifest.format_version != 1 || manifest.schema_version > crate::migrations::CURRENT_VERSION
    {
        return Err("Unsupported backup version. Update CowWorker.".into());
    }
    for (relative, expected) in &manifest.files {
        let path = Path::new(relative);
        if path
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
            || !["cowworker.db", "documents", "sources", "company-assets"].contains(
                &path
                    .components()
                    .next()
                    .ok_or("Empty asset path")?
                    .as_os_str()
                    .to_str()
                    .unwrap_or(""),
            )
        {
            return Err("Unsafe backup asset path.".into());
        }
        if hash(&directory.join(path))? != *expected {
            return Err(format!("Backup checksum mismatch: {relative}"));
        }
    }
    if !manifest.files.contains_key("cowworker.db") {
        return Err("Backup database is missing.".into());
    }
    let db = Connection::open_with_flags(
        directory.join("cowworker.db"),
        OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .map_err(err)?;
    integrity(&db)?;
    let version: i64 = db
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .map_err(err)?;
    if version != manifest.schema_version || counts(&db)? != manifest.counts {
        return Err("Backup metadata does not match its database.".into());
    }
    if references(&db)?
        .iter()
        .any(|p| !manifest.files.contains_key(p))
    {
        return Err("Backup manifest omits a referenced asset.".into());
    }
    Ok(manifest)
}

pub fn restore(backup: &Path, destination: &Path) -> Result<Manifest> {
    let manifest = verify(backup)?;
    if destination.exists() {
        return Err(
            "Restore requires a new directory. Close storage before switching workspaces.".into(),
        );
    }
    fs::create_dir_all(destination).map_err(err)?;
    for relative in manifest
        .files
        .keys()
        .chain(std::iter::once(&"manifest.json".to_string()))
    {
        let target = destination.join(relative);
        fs::create_dir_all(target.parent().ok_or("Invalid restore path")?).map_err(err)?;
        fs::copy(backup.join(relative), target).map_err(err)?;
    }
    verify(destination)
}

impl Store {
    pub fn backup(&self, destination: &Path) -> Result<Manifest> {
        create(&self.db, &self.path, destination)
    }
}
