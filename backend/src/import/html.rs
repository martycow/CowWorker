use serde_json::Value;

fn visible(document: &scraper::Html) -> String {
    document
        .root_element()
        .descendants()
        .filter_map(|node| {
            if node.ancestors().any(|a| {
                a.value().as_element().is_some_and(|e| {
                    ["script", "style", "noscript", "nav", "footer"].contains(&e.name())
                })
            }) {
                return None;
            }
            node.value()
                .as_text()
                .map(|t| t.text.trim().to_string())
                .filter(|t| !t.is_empty())
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn posting(value: &Value) -> Option<&Value> {
    match value {
        Value::Array(values) => values.iter().find_map(posting),
        Value::Object(values) => {
            let kind = &value["@type"];
            if kind.as_str() == Some("JobPosting")
                || kind
                    .as_array()
                    .is_some_and(|types| types.iter().any(|v| v == "JobPosting"))
            {
                Some(value)
            } else {
                values.values().find_map(posting)
            }
        }
        _ => None,
    }
}

pub fn extract(html: &str) -> Result<String, String> {
    let document = scraper::Html::parse_document(html);
    let selector = scraper::Selector::parse("script[type='application/ld+json']")
        .map_err(|e| e.to_string())?;
    for script in document.select(&selector) {
        let Ok(json) = serde_json::from_str::<Value>(&script.text().collect::<String>()) else {
            continue;
        };
        let Some(job) = posting(&json) else {
            continue;
        };
        let Some(description) = job["description"].as_str().filter(|d| !d.trim().is_empty()) else {
            continue;
        };
        let mut text = String::from("Job posting\n");
        for (label, value) in [
            ("Title", &job["title"]),
            ("Company", &job["hiringOrganization"]["name"]),
            ("Employment type", &job["employmentType"]),
        ] {
            if let Some(value) = value.as_str() {
                text.push_str(&format!("{label}: {value}\n"));
            }
        }
        if job["jobLocationType"] == "TELECOMMUTE" {
            text.push_str("Work mode: Remote\n");
        }
        let location = if job["jobLocation"].is_array() {
            &job["jobLocation"][0]
        } else {
            &job["jobLocation"]
        };
        let location = ["addressLocality", "addressRegion", "addressCountry"]
            .iter()
            .filter_map(|key| location["address"][key].as_str())
            .collect::<Vec<_>>()
            .join(", ");
        if !location.is_empty() {
            text.push_str(&format!("Location: {location}\n"));
        }
        text.push('\n');
        text.push_str(&visible(&scraper::Html::parse_document(description)));
        return Ok(text);
    }
    let text = visible(&document);
    if text.trim().is_empty() {
        return Err(
            "This page has no readable text. Paste the vacancy text or screenshots instead.".into(),
        );
    }
    Ok(text)
}
