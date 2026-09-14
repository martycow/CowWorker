use rusqlite::Connection;

pub const CURRENT_VERSION: i64 = 9;
pub const PRESERVED_TABLES: [&str; 6] = [
    "vacancies",
    "documents",
    "document_versions",
    "applications",
    "application_events",
    "application_documents",
];

fn record_counts(db: &Connection) -> Result<Vec<(&'static str, i64)>, String> {
    PRESERVED_TABLES
        .iter()
        .map(|&table| {
            db.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
                .map(|count| (table, count))
                .map_err(|e| e.to_string())
        })
        .collect()
}

pub fn run(db: &Connection) -> Result<(), String> {
    // Foreign keys must be disabled before the transaction when rebuilding a parent table.
    db.execute_batch("PRAGMA foreign_keys=OFF;")
        .map_err(|e| e.to_string())?;
    let result = (|| {
        let tx = db.unchecked_transaction().map_err(|e| e.to_string())?;
        let mut version: i64 = tx
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        let original_counts = if version > 0 && version < CURRENT_VERSION {
            record_counts(&tx)?
        } else {
            Vec::new()
        };
        if version == 0 {
            tx.execute_batch(include_str!("../schema.sql"))
                .map_err(|e| e.to_string())?;
            version = 1;
        }
        if version == 1 {
            tx.execute_batch(include_str!("002.sql"))
                .map_err(|e| e.to_string())?;
            version = 2;
        }
        if version == 2 {
            tx.execute_batch(include_str!("003.sql"))
                .map_err(|e| e.to_string())?;
            version = 3;
        }
        if version == 3 {
            tx.execute_batch(include_str!("004.sql"))
                .map_err(|e| e.to_string())?;
            version = 4;
        }
        if version == 4 {
            tx.execute_batch(include_str!("005.sql"))
                .map_err(|e| e.to_string())?;
            version = 5;
        }
        if version == 5 {
            tx.execute_batch(include_str!("006.sql"))
                .map_err(|e| e.to_string())?;
            version = 6;
        }
        if version == 6 {
            tx.execute_batch(include_str!("007.sql"))
                .map_err(|e| e.to_string())?;
            version = 7;
        }
        if version == 7 {
            tx.execute_batch(include_str!("008.sql"))
                .map_err(|e| e.to_string())?;
            version = 8;
        }
        if version == 8 {
            tx.execute_batch(include_str!("009.sql"))
                .map_err(|e| e.to_string())?;
        }
        let broken: i64 = tx
            .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |r| {
                r.get(0)
            })
            .map_err(|e| e.to_string())?;
        if broken != 0 {
            return Err(
                "Migration failed its foreign-key check. Original data was retained.".into(),
            );
        }
        for (table, count) in original_counts {
            let current: i64 = tx
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
                .map_err(|e| e.to_string())?;
            if current != count {
                return Err(format!(
                    "Migration changed the record count in {table}. The upgrade was rolled back."
                ));
            }
        }
        tx.commit().map_err(|e| e.to_string())
    })();
    db.execute_batch("PRAGMA foreign_keys=ON;")
        .map_err(|e| e.to_string())?;
    result
}
