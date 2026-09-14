#[cfg(windows)]
pub fn extract(bytes: &[u8]) -> Result<String, String> {
    use windows::{
        Graphics::Imaging::BitmapDecoder,
        Media::Ocr::OcrEngine,
        Storage::Streams::{DataWriter, InMemoryRandomAccessStream},
    };
    let reader = image::ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| e.to_string())?;
    let (width, height) = reader
        .into_dimensions()
        .map_err(|_| "Unsupported or corrupt screenshot.")?;
    if width == 0 || height == 0 || u64::from(width) * u64::from(height) > 24_000_000 {
        return Err("Screenshot exceeds 24 megapixels. Split it into smaller images.".into());
    }
    let max = OcrEngine::MaxImageDimension().map_err(|e| e.to_string())?;
    let image = image::load_from_memory(bytes).map_err(|e| e.to_string())?;
    let image = if width > max || height > max {
        image.resize(max, max, image::imageops::FilterType::Lanczos3)
    } else {
        image
    };
    let mut encoded = std::io::Cursor::new(Vec::new());
    image
        .write_to(&mut encoded, image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    let result = (|| -> windows::core::Result<String> {
        let stream = InMemoryRandomAccessStream::new()?;
        let writer = DataWriter::CreateDataWriter(&stream)?;
        writer.WriteBytes(encoded.get_ref())?;
        writer.StoreAsync()?.join()?;
        stream.Seek(0)?;
        let bitmap = BitmapDecoder::CreateAsync(&stream)?
            .join()?
            .GetSoftwareBitmapAsync()?
            .join()?;
        let engine = OcrEngine::TryCreateFromUserProfileLanguages()?;
        let result = engine.RecognizeAsync(&bitmap)?.join()?;
        let mut lines = Vec::new();
        for line in result.Lines()? {
            lines.push(line.Text()?.to_string());
        }
        Ok(lines.join("\n"))
    })();
    let text=result.map_err(|e|format!("Windows OCR could not recognize this image ({e}). Check that a Windows language pack with OCR is installed, or enter a manual transcript."))?;
    if text.trim().is_empty() {
        return Err("No readable text found in this screenshot. Try a clearer image or enter a manual transcript.".into());
    }
    Ok(text)
}
#[cfg(not(windows))]
pub fn extract(_: &[u8]) -> Result<String, String> {
    Err("Local screenshot OCR currently requires Windows. Enter a manual transcript.".into())
}
#[cfg(windows)]
pub fn scanned_pdf(bytes: &[u8]) -> Result<String, String> {
    use windows::{
        Data::Pdf::{PdfDocument, PdfPageRenderOptions},
        Graphics::Imaging::BitmapDecoder,
        Media::Ocr::OcrEngine,
        Storage::Streams::{DataWriter, InMemoryRandomAccessStream},
    };
    let result = (|| -> windows::core::Result<String> {
        let source = InMemoryRandomAccessStream::new()?;
        let writer = DataWriter::CreateDataWriter(&source)?;
        writer.WriteBytes(bytes)?;
        writer.StoreAsync()?.join()?;
        source.Seek(0)?;
        let pdf = PdfDocument::LoadFromStreamAsync(&source)?.join()?;
        if pdf.PageCount()? > 20 {
            return Err(windows::core::Error::new(
                windows::core::HRESULT(0x80070057u32 as i32),
                "Scanned PDF has more than 20 pages. Split it into smaller files.",
            ));
        }
        let engine = OcrEngine::TryCreateFromUserProfileLanguages()?;
        let mut pages = Vec::new();
        for number in 0..pdf.PageCount()? {
            let page = pdf.GetPage(number)?;
            let size = page.Size()?;
            let width = 1600f32.min(2400.0 * size.Width / size.Height).max(1.0) as u32;
            let options = PdfPageRenderOptions::new()?;
            options.SetDestinationWidth(width)?;
            let stream = InMemoryRandomAccessStream::new()?;
            page.RenderWithOptionsToStreamAsync(&stream, &options)?
                .join()?;
            stream.Seek(0)?;
            let bitmap = BitmapDecoder::CreateAsync(&stream)?
                .join()?
                .GetSoftwareBitmapAsync()?
                .join()?;
            let result = engine.RecognizeAsync(&bitmap)?.join()?;
            let mut lines = Vec::new();
            for line in result.Lines()? {
                lines.push(line.Text()?.to_string());
            }
            pages.push(lines.join("\n"));
            page.Close()?;
        }
        Ok(pages.join("\n\n"))
    })();
    result.map_err(|e| {
        format!("Scanned PDF OCR failed: {e}. Try clearer page images or a text copy.")
    })
}
#[cfg(not(windows))]
pub fn scanned_pdf(_: &[u8]) -> Result<String, String> {
    Err("Scanned PDF OCR currently requires Windows. Import a text PDF or transcript.".into())
}
