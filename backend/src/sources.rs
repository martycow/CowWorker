use crate::Store;
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{fs, io::Write};

type Result<T> = std::result::Result<T, String>;
fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SourceAsset {
    pub id: String,
    pub name: String,
    pub media_type: String,
    pub sha256: String,
    pub byte_length: i64,
    pub source_url: Option<String>,
    pub created_at: String,
}
impl Store {
    pub fn acquire_source(
        &mut self,
        name: &str,
        media_type: &str,
        bytes: &[u8],
        url: Option<&str>,
    ) -> Result<SourceAsset> {
        if bytes.is_empty() || bytes.len() > 25_000_000 {
            return Err("Sources must contain 1 byte to 25 MB.".into());
        }
        if name.len() > 1000 || media_type.len() > 200 {
            return Err("Source metadata exceeds its limit.".into());
        }
        if let Some(url) = url {
            let parsed = url::Url::parse(url).map_err(err)?;
            if !["http", "https"].contains(&parsed.scheme())
                || parsed.host_str().is_none()
                || !parsed.username().is_empty()
                || parsed.password().is_some()
            {
                return Err("Invalid source URL.".into());
            }
        }
        let id = uuid::Uuid::new_v4().to_string();
        let file_name = format!("{id}.bin");
        let directory = self.path.join("sources");
        fs::create_dir_all(&directory).map_err(err)?;
        let temporary = directory.join(format!("{id}.tmp"));
        let target = directory.join(&file_name);
        let mut file = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary)
            .map_err(err)?;
        file.write_all(bytes).map_err(err)?;
        file.sync_all().map_err(err)?;
        drop(file);
        let sha256 = format!("{:x}", Sha256::digest(bytes));
        if crate::backup::hash(&temporary)? != sha256 {
            return Err("Source hash verification failed.".into());
        }
        fs::rename(&temporary, &target).map_err(err)?;
        let source = SourceAsset {
            id,
            name: name.into(),
            media_type: media_type.into(),
            sha256,
            byte_length: bytes.len() as i64,
            source_url: url.map(str::to_string),
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        self.db
            .execute(
                "INSERT INTO sources VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
                params![
                    source.id,
                    file_name,
                    source.name,
                    source.media_type,
                    source.sha256,
                    source.byte_length,
                    source.source_url,
                    source.created_at
                ],
            )
            .map_err(err)?;
        Ok(source)
    }
    pub fn source_bytes(&self, id: &str) -> Result<Vec<u8>> {
        let (file, hash): (String, String) = self
            .db
            .query_row(
                "SELECT file_name,sha256 FROM sources WHERE id=?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .map_err(err)?;
        if uuid::Uuid::parse_str(id).is_err() || file != format!("{id}.bin") {
            return Err("Invalid source file reference.".into());
        }
        let path = self.path.join("sources").join(file);
        if crate::backup::hash(&path)? != hash {
            return Err("Source checksum mismatch. Restore the original from backup.".into());
        }
        fs::read(path).map_err(err)
    }
    pub fn source_snapshot(
        &mut self,
        source: &str,
        text: &str,
        tool: &str,
        version: &str,
    ) -> Result<String> {
        if text.len() > 1_000_000 {
            return Err(
                "Extraction exceeds the 1 MB text limit. Original remains preserved.".into(),
            );
        }
        let id = uuid::Uuid::new_v4().to_string();
        self.db
            .execute(
                "INSERT INTO source_snapshots VALUES(?1,?2,?3,?4,?5,'[]',?6)",
                params![
                    id,
                    source,
                    tool,
                    version,
                    text,
                    chrono::Utc::now().to_rfc3339()
                ],
            )
            .map_err(err)?;
        Ok(id)
    }
    pub fn entity_sources(&self, entity: &str) -> Result<Vec<SourceAsset>> {
        let mut stmt=self.db.prepare("SELECT s.id,s.name,s.media_type,s.sha256,s.byte_length,s.source_url,s.created_at FROM sources s JOIN entity_sources e ON e.source_id=s.id WHERE e.entity_id=?1 ORDER BY s.created_at DESC LIMIT 100").map_err(err)?;
        let rows = stmt
            .query_map([entity], |r| {
                Ok(SourceAsset {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    media_type: r.get(2)?,
                    sha256: r.get(3)?,
                    byte_length: r.get(4)?,
                    source_url: r.get(5)?,
                    created_at: r.get(6)?,
                })
            })
            .map_err(err)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(err)?;
        Ok(rows)
    }
    pub fn link_source(&mut self, kind: &str, entity: &str, source: &str) -> Result<()> {
        let table = match kind {
            "vacancy" => "vacancies",
            "document" => "documents",
            _ => return Err("Unsupported source owner.".into()),
        };
        let exists: Option<String> = self
            .db
            .query_row(
                &format!("SELECT id FROM {table} WHERE id=?1"),
                [entity],
                |r| r.get(0),
            )
            .optional()
            .map_err(err)?;
        if exists.is_none() {
            return Err("Source owner no longer exists.".into());
        }
        self.db
            .execute(
                "INSERT OR IGNORE INTO entity_sources VALUES(?1,?2,?3)",
                params![kind, entity, source],
            )
            .map_err(err)?;
        Ok(())
    }
    pub fn reconcile_assets(&self) -> Result<Vec<String>> {
        let mut quarantined = vec![];
        for (directory, table) in [("documents", "document_versions"), ("sources", "sources")] {
            let root = self.path.join(directory);
            if !root.exists() {
                continue;
            }
            for entry in fs::read_dir(&root).map_err(err)? {
                let entry = entry.map_err(err)?;
                if !entry.file_type().map_err(err)?.is_file() {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().to_string();
                let exists: bool = self
                    .db
                    .query_row(
                        &format!("SELECT EXISTS(SELECT 1 FROM {table} WHERE file_name=?1)"),
                        [&name],
                        |r| r.get(0),
                    )
                    .map_err(err)?;
                if !exists {
                    let dest = self
                        .path
                        .join("quarantine")
                        .join(uuid::Uuid::new_v4().to_string());
                    fs::create_dir_all(&dest).map_err(err)?;
                    fs::rename(entry.path(), dest.join(&name)).map_err(err)?;
                    quarantined.push(format!("{directory}/{name}"));
                }
            }
        }
        Ok(quarantined)
    }
}
