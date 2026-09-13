use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Vacancy {
    pub id: String,
    pub title: String,
    pub company: String,
    pub location: String,
    pub work_mode: String,
    pub source_url: String,
    pub description: String,
    pub notes: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VacancyInput {
    pub id: Option<String>,
    pub title: String,
    pub company: String,
    pub location: String,
    pub work_mode: String,
    pub source_url: String,
    pub description: String,
    pub notes: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub id: String,
    pub title: String,
    pub kind: String,
    pub versions: Vec<DocumentVersion>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DocumentVersion {
    pub id: String,
    pub number: i64,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DocumentInput {
    pub id: Option<String>,
    pub title: String,
    pub kind: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Application {
    pub id: String,
    pub vacancy_id: String,
    pub stage: String,
    pub next_action: String,
    pub due_date: String,
    pub notes: String,
    pub document_version_ids: Vec<String>,
    pub submitted_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub events: Vec<ApplicationEvent>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationEvent {
    pub id: String,
    pub stage: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ApplicationInput {
    pub id: String,
    pub stage: String,
    pub next_action: String,
    pub due_date: String,
    pub notes: String,
    pub document_version_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Profile {
    pub name: String,
    pub headline: String,
    pub email: String,
    pub summary: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    pub schema_version: i64,
    pub vacancies: Vec<Vacancy>,
    pub documents: Vec<Document>,
    pub applications: Vec<Application>,
    pub profile: Profile,
}
