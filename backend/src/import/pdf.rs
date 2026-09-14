use std::collections::BTreeMap;
fn stream(dictionary: &str, data: &[u8]) -> Vec<u8> {
    let mut bytes = format!("<< {dictionary} /Length {} >>\nstream\n", data.len()).into_bytes();
    bytes.extend_from_slice(data);
    bytes.extend_from_slice(b"\nendstream");
    bytes
}
pub fn export(text: &str) -> Result<Vec<u8>, String> {
    let font = include_bytes!("../../assets/NotoSans-Regular.ttf");
    let face = ttf_parser::Face::parse(font, 0).map_err(|_| "PDF font could not be loaded.")?;
    let scale = 1000.0 / face.units_per_em() as f64;
    let mut glyphs = BTreeMap::new();
    let mut lines = vec![];
    for paragraph in text.replace('\t', "    ").lines() {
        let mut line = String::new();
        let mut width = 0.0;
        for ch in paragraph.chars() {
            if ch == '\r' {
                continue;
            }
            let gid=face.glyph_index(ch).ok_or_else(||format!("PDF font does not support U+{:04X}. Export DOCX or text to retain this character.",ch as u32))?;
            let advance = face.glyph_hor_advance(gid).unwrap_or(500) as f64 * scale;
            glyphs.insert(gid.0, (ch, advance));
            if width + advance * 11.0 / 1000.0 > 504.0 && !line.is_empty() {
                if let Some(space) = line.rfind(' ').filter(|&position| position > 0) {
                    let rest = line[space + 1..].to_string();
                    lines.push(line[..space].to_string());
                    line = rest;
                    width = line
                        .chars()
                        .map(|c| {
                            face.glyph_index(c)
                                .and_then(|g| face.glyph_hor_advance(g))
                                .unwrap_or(500) as f64
                                * scale
                                * 11.0
                                / 1000.0
                        })
                        .sum();
                } else {
                    lines.push(line);
                    line = String::new();
                    width = 0.0;
                }
            }
            line.push(ch);
            width += advance * 11.0 / 1000.0;
        }
        lines.push(line);
    }
    if lines.len() > 10000 {
        return Err("PDF export exceeds 10000 lines.".into());
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    let mut cmap=String::from("/CIDInit /ProcSet findresource begin\n12 dict begin\nbegincmap\n/CIDSystemInfo << /Registry (Adobe) /Ordering (UCS) /Supplement 0 >> def\n/CMapName /CowWorkerUnicode def\n/CMapType 2 def\n1 begincodespacerange\n<0000> <FFFF>\nendcodespacerange\n");
    let entries = glyphs.iter().collect::<Vec<_>>();
    for group in entries.chunks(100) {
        cmap.push_str(&format!("{} beginbfchar\n", group.len()));
        for (gid, (ch, _)) in group {
            let mut units = [0u16; 2];
            let hex = ch
                .encode_utf16(&mut units)
                .iter()
                .map(|u| format!("{u:04X}"))
                .collect::<String>();
            cmap.push_str(&format!("<{gid:04X}> <{hex}>\n"));
        }
        cmap.push_str("endbfchar\n");
    }
    cmap.push_str("endcmap\nCMapName currentdict /CMap defineresource pop\nend\nend");
    let widths = glyphs
        .iter()
        .map(|(id, (_, width))| format!("{id} [{}]", *width as i64))
        .collect::<Vec<_>>()
        .join(" ");
    let bbox = face.global_bounding_box();
    let mut objects:Vec<Vec<u8>>=vec![b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),vec![],b"<< /Type /Font /Subtype /Type0 /BaseFont /NotoSans /Encoding /Identity-H /DescendantFonts [4 0 R] /ToUnicode 7 0 R >>".to_vec(),format!("<< /Type /Font /Subtype /CIDFontType2 /BaseFont /NotoSans /CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> /FontDescriptor 5 0 R /CIDToGIDMap /Identity /DW 500 /W [{widths}] >>").into_bytes(),format!("<< /Type /FontDescriptor /FontName /NotoSans /Flags 32 /FontBBox [{} {} {} {}] /ItalicAngle 0 /Ascent {} /Descent {} /CapHeight 714 /StemV 80 /FontFile2 6 0 R >>",bbox.x_min as f64*scale,bbox.y_min as f64*scale,bbox.x_max as f64*scale,bbox.y_max as f64*scale,face.ascender() as f64*scale,face.descender() as f64*scale).into_bytes(),stream(&format!("/Length1 {}",font.len()),font),stream("",cmap.as_bytes())];
    let mut kids = vec![];
    for group in lines.chunks(44) {
        let page = objects.len() + 1;
        let content = page + 1;
        kids.push(format!("{page} 0 R"));
        objects.push(format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 3 0 R >> >> /Contents {content} 0 R >>").into_bytes());
        let mut commands = String::from("BT /F1 11 Tf 54 738 Td 15 TL\n");
        for line in group {
            let encoded = line
                .chars()
                .map(|c| format!("{:04X}", face.glyph_index(c).expect("validated glyph").0))
                .collect::<String>();
            commands.push_str(&format!("<{encoded}> Tj T*\n"));
        }
        commands.push_str("ET");
        objects.push(stream("", commands.as_bytes()));
    }
    objects[1] = format!(
        "<< /Type /Pages /Kids [{}] /Count {} >>",
        kids.join(" "),
        kids.len()
    )
    .into_bytes();
    let mut pdf = b"%PDF-1.7\n%\xE2\xE3\xCF\xD3\n".to_vec();
    let mut offsets = vec![0];
    for (i, obj) in objects.iter().enumerate() {
        offsets.push(pdf.len());
        pdf.extend_from_slice(format!("{} 0 obj\n", i + 1).as_bytes());
        pdf.extend_from_slice(obj);
        pdf.extend_from_slice(b"\nendobj\n");
    }
    let xref = pdf.len();
    pdf.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", offsets.len()).as_bytes());
    for offset in offsets.iter().skip(1) {
        pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    pdf.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            offsets.len()
        )
        .as_bytes(),
    );
    Ok(pdf)
}
