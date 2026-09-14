use std::io::{Cursor, Read, Write};
type Result<T> = std::result::Result<T, String>;
fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}
pub fn extract(bytes: &[u8], media_type: &str) -> Result<String> {
    let text = match media_type {
        "text/html" => {
            let html = String::from_utf8(bytes.to_vec())
                .map_err(|_| "Page is not UTF-8. Paste its readable text instead.".to_string())?;
            super::html::extract(&html)?
        }
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => {
            extract_docx(bytes)?
        }
        "application/pdf" => {
            if !bytes.starts_with(b"%PDF-") {
                return Err("Invalid PDF header.".into());
            }
            let text = pdf_extract::extract_text_from_mem(bytes).map_err(|_| {
                "PDF is encrypted, corrupt, or unsupported. Upload a text copy.".to_string()
            })?;
            if text.trim().is_empty() {
                super::ocr::scanned_pdf(bytes)?
            } else {
                text
            }
        }
        "image/png" | "image/jpeg" | "image/webp" => super::ocr::extract(bytes)?,
        "text/plain" | "text/markdown" => String::from_utf8(bytes.to_vec())
            .map_err(|_| "Text must use UTF-8 encoding.".to_string())?,
        _ => return Err("Unsupported file type. The original is preserved for review.".into()),
    };
    if text.is_empty() || text.contains('\0') || text.len() > 1_000_000 {
        return Err(
            "Extracted text is empty, binary, or exceeds 1 MB. Original remains preserved.".into(),
        );
    }
    Ok(text)
}
pub fn extract_docx(bytes: &[u8]) -> Result<String> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|_| "DOCX is corrupt or encrypted.".to_string())?;
    if archive.len() > 2000 {
        return Err("DOCX contains too many parts.".into());
    }
    let mut total = 0u64;
    for i in 0..archive.len() {
        let file = archive.by_index(i).map_err(err)?;
        total = total.checked_add(file.size()).ok_or("DOCX size overflow")?;
        if total > 50_000_000 || file.size() > 20_000_000 || file.name().ends_with("vbaProject.bin")
        {
            return Err("DOCX exceeds its decompression limit or contains macros.".into());
        }
    }
    let mut file = archive
        .by_name("word/document.xml")
        .map_err(|_| "DOCX document body is missing.".to_string())?;
    let mut xml = String::new();
    file.by_ref()
        .take(20_000_001)
        .read_to_string(&mut xml)
        .map_err(err)?;
    if xml.len() > 20_000_000 {
        return Err("DOCX body exceeds its limit.".into());
    }
    let mut reader = quick_xml::Reader::from_str(&xml);
    let mut text = String::new();
    let mut in_text = false;
    loop {
        use quick_xml::events::Event;
        match reader.read_event().map_err(err)? {
            Event::Start(e) if e.local_name().as_ref() == b"t" => in_text = true,
            Event::End(e) => match e.local_name().as_ref() {
                b"t" => in_text = false,
                b"p" => text.push('\n'),
                _ => (),
            },
            Event::Empty(e) => match e.local_name().as_ref() {
                b"tab" => text.push('\t'),
                b"br" | b"cr" => text.push('\n'),
                _ => (),
            },
            Event::Text(e) if in_text => {
                text.push_str(&quick_xml::escape::unescape(&e.decode().map_err(err)?).map_err(err)?)
            }
            Event::GeneralRef(e) if in_text => text.push_str(
                &quick_xml::escape::unescape(&format!("&{};", e.decode().map_err(err)?))
                    .map_err(err)?,
            ),
            Event::DocType(_) => return Err("DOCX external declarations are unsupported.".into()),
            Event::Eof => break,
            _ => (),
        }
        if text.len() > 1_000_000 {
            return Err("DOCX text exceeds 1 MB.".into());
        }
    }
    Ok(text)
}
pub fn export_docx(text: &str) -> Result<Vec<u8>> {
    let mut archive = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    let body = text
        .split('\n')
        .map(|line| {
            format!(
                "<w:p><w:r><w:t xml:space=\"preserve\">{}</w:t></w:r></w:p>",
                quick_xml::escape::escape(line.trim_end_matches('\r'))
            )
        })
        .collect::<String>();
    let xml=format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?><w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"><w:body>{body}<w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/><w:pgMar w:top=\"1080\" w:right=\"1080\" w:bottom=\"1080\" w:left=\"1080\"/></w:sectPr></w:body></w:document>");
    for (name,data) in [("[Content_Types].xml","<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"><Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/><Default Extension=\"xml\" ContentType=\"application/xml\"/><Override PartName=\"/word/document.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml\"/></Types>"),("_rels/.rels","<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"word/document.xml\"/></Relationships>"),("word/document.xml",xml.as_str())]{archive.start_file(name,options).map_err(err)?;archive.write_all(data.as_bytes()).map_err(err)?;}
    Ok(archive.finish().map_err(err)?.into_inner())
}
