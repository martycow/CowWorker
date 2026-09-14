#[test]
#[ignore = "Requires the packaged Windows debug executable; run after the native build."]
fn packaged_parser_handles_pdf_docx_and_corrupt_input() {
    let executable = std::env::var_os("COWWORKER_PARSER_EXE")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../target/debug/cowworker.exe")
        });
    assert!(executable.exists());
    let dir = tempfile::tempdir().unwrap();
    let content = "Точный текст • Engineer & <evidence>";
    for (name, media, bytes) in [
        (
            "fixture.pdf",
            "application/pdf",
            cowworker_core::import::pdf::export(content).unwrap(),
        ),
        (
            "fixture.docx",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            cowworker_core::import::formats::export_docx(content).unwrap(),
        ),
    ] {
        let input = dir.path().join(name);
        std::fs::write(&input, bytes).unwrap();
        let text = cowworker_core::import::process::extract(&executable, &input, media, dir.path())
            .unwrap();
        assert!(text.contains("Точный текст"));
        assert!(text.contains("& <evidence>"));
    }
    let input = dir.path().join("bad.pdf");
    std::fs::write(&input, b"%PDF-corrupt").unwrap();
    assert!(cowworker_core::import::process::extract(
        &executable,
        &input,
        "application/pdf",
        dir.path()
    )
    .is_err());
    for (name, media, bytes) in [
        (
            "screenshot.png",
            "image/png",
            include_bytes!("fixtures/vacancy-screen-1.png").as_slice(),
        ),
        (
            "scan.pdf",
            "application/pdf",
            include_bytes!("fixtures/scanned-vacancy.pdf").as_slice(),
        ),
    ] {
        let input = dir.path().join(name);
        std::fs::write(&input, bytes).unwrap();
        let text = cowworker_core::import::process::extract(&executable, &input, media, dir.path())
            .unwrap();
        assert!(text.contains("Morning Example"));
    }
}
