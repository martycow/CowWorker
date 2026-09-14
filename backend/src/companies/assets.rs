use crate::Store;
use rusqlite::{params, OptionalExtension};
use std::io::Cursor;
impl Store {
    pub fn save_company_logo(
        &mut self,
        company: &str,
        expected: i64,
        name: &str,
        bytes: &[u8],
    ) -> Result<(), String> {
        if bytes.len() > 2_000_000 {
            return Err("Company logos must be smaller than 2 MB.".into());
        }
        let reader = image::ImageReader::new(Cursor::new(bytes))
            .with_guessed_format()
            .map_err(|e| e.to_string())?;
        let dimensions = reader
            .into_dimensions()
            .map_err(|_| "Unsupported or corrupt company image.")?;
        if dimensions.0 > 2048 || dimensions.1 > 2048 {
            return Err("Company logos must be at most 2048 pixels on each side.".into());
        }
        let decoded =
            image::load_from_memory(bytes).map_err(|_| "Company image could not be decoded.")?;
        let mut normalized = Cursor::new(Vec::new());
        decoded
            .write_to(&mut normalized, image::ImageFormat::Png)
            .map_err(|e| e.to_string())?;
        let original = self.acquire_source(name, "application/octet-stream", bytes, None)?;
        let sanitized =
            self.acquire_source("Company logo.png", "image/png", normalized.get_ref(), None)?;
        let tx = self
            .db
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(|e| e.to_string())?;
        if tx.execute("UPDATE companies SET revision=revision+1 WHERE id=?1 AND revision=?2 AND merged_into IS NULL",params![company,expected]).map_err(|e|e.to_string())?!=1{return Err("Revision conflict. Reload the company before replacing its logo.".into());}
        tx.execute(
            "INSERT INTO company_assets VALUES(?1,?2,?3,'logo',?4)",
            params![
                uuid::Uuid::new_v4().to_string(),
                company,
                sanitized.id,
                chrono::Utc::now().timestamp()
            ],
        )
        .map_err(|e| e.to_string())?;
        for source in [&original.id, &sanitized.id] {
            tx.execute(
                "INSERT OR IGNORE INTO entity_sources VALUES('company',?1,?2)",
                params![company, source],
            )
            .map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())
    }
    pub fn company_logo(&self, company: &str) -> Result<Option<Vec<u8>>, String> {
        let source:Option<String>=self.db.query_row("SELECT source_id FROM company_assets WHERE company_id=?1 AND kind='logo' ORDER BY created_at DESC,rowid DESC LIMIT 1",[company],|r|r.get(0)).optional().map_err(|e|e.to_string())?;
        source.map(|id| self.source_bytes(&id)).transpose()
    }
}
