use crate::{DocumentInput, Store};
use rusqlite::params;
impl Store {
    pub fn prepare_job_document(
        &mut self,
        vacancy: &str,
        expected: i64,
        resume_version: &str,
        kind: &str,
    ) -> Result<String, String> {
        if !["resume", "cover-letter"].contains(&kind) {
            return Err("Choose Resume or Cover Letter.".into());
        }
        self.db
            .execute_batch("BEGIN IMMEDIATE")
            .map_err(|e| e.to_string())?;
        let result = (|| {
            let (title, company): (String, String) = self
                .db
                .query_row(
                    "SELECT title,company FROM vacancies WHERE id=?1 AND revision=?2",
                    params![vacancy, expected],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .map_err(|_| {
                    "Vacancy changed. Reload it before preparing documents.".to_string()
                })?;
            let resume:String=self.db.query_row("SELECT d.id FROM documents d JOIN document_versions v ON v.document_id=d.id WHERE v.id=?1 AND d.kind='resume'",[resume_version],|r|r.get(0)).map_err(|_|"Select a saved resume version. Correct its category in Documents if needed.".to_string())?;
            let content = if kind == "resume" {
                self.document_content(resume_version)?
            } else {
                let profile = self.workspace()?.profile;
                format!("Dear hiring team at {company},\n\nI am applying for the {title} position.\n\n[Explain why this role interests you.]\n\n[Describe relevant experience from your resume, with examples you can support.]\n\n[Add a closing paragraph and your contact details.]\n\nSincerely,\n{}",if profile.name.trim().is_empty(){"[Your name]"}else{&profile.name})
            };
            let name = format!(
                "{} — {} · {}",
                if kind == "resume" {
                    "Resume"
                } else {
                    "Cover letter"
                },
                title,
                company
            )
            .chars()
            .take(140)
            .collect::<String>();
            let document = self.save_document(DocumentInput {
                id: None,
                expected_revision: None,
                title: name,
                kind: kind.into(),
                content,
            })?;
            self.db
                .execute(
                    "INSERT INTO document_job_context VALUES(?1,?2,?3)",
                    params![document, vacancy, resume_version],
                )
                .map_err(|e| e.to_string())?;
            self.db.execute("INSERT OR IGNORE INTO entity_sources SELECT 'document',?1,source_id FROM entity_sources WHERE entity_type='document' AND entity_id=?2",params![document,resume]).map_err(|e|e.to_string())?;
            Ok(document)
        })();
        match result {
            Ok(id) => {
                self.db.execute_batch("COMMIT").map_err(|e| e.to_string())?;
                Ok(id)
            }
            Err(e) => {
                let _ = self.db.execute_batch("ROLLBACK");
                Err(e)
            }
        }
    }
}
