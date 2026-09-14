use cowworker_core::*;
use rusqlite::Connection;

#[test]
fn upgrade_rolls_back_if_a_legacy_trigger_removes_records() {
    let dir = tempfile::tempdir().unwrap();
    let db = Connection::open(dir.path().join("cowworker.db")).unwrap();
    db.execute_batch(include_str!("../src/schema.sql")).unwrap();
    db.execute_batch("INSERT INTO vacancies VALUES ('v','Role','Company','','Remote','','','','saved','now','now'); INSERT INTO applications VALUES ('a','v','preparing','','','',NULL,'now','now'); CREATE TRIGGER legacy_side_effect AFTER UPDATE ON vacancies BEGIN DELETE FROM applications; END;").unwrap();
    drop(db);
    let error = Store::open(dir.path())
        .err()
        .expect("upgrade must reject record removal");
    assert!(error.contains("record count"));
    let db = Connection::open(dir.path().join("cowworker.db")).unwrap();
    assert_eq!(
        db.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        db.query_row("SELECT COUNT(*) FROM applications", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        1
    );
}

#[test]
fn simultaneous_legacy_open_preserves_records() {
    for _ in 0..12 {
        let dir = tempfile::tempdir().unwrap();
        let db = Connection::open(dir.path().join("cowworker.db")).unwrap();
        db.execute_batch(include_str!("../src/schema.sql")).unwrap();
        db.execute_batch("PRAGMA journal_mode=WAL; INSERT INTO vacancies VALUES ('v','Role','Company','','Remote','','','','saved','now','now'); INSERT INTO applications VALUES ('a','v','preparing','','','',NULL,'now','now');").unwrap();
        drop(db);
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(4));
        let mut threads = vec![];
        for _ in 0..4 {
            let path = dir.path().to_path_buf();
            let barrier = barrier.clone();
            threads.push(std::thread::spawn(move || {
                barrier.wait();
                if let Ok(store) = Store::open(path) {
                    let data = store.workspace().unwrap();
                    assert_eq!(data.vacancies.len(), 1);
                    assert_eq!(data.applications.len(), 1);
                }
            }));
        }
        for thread in threads {
            thread.join().unwrap();
        }
        let store = Store::open(dir.path()).unwrap();
        assert_eq!(store.workspace().unwrap().vacancies.len(), 1);
    }
}

#[test]
#[ignore = "Child process fixture, invoked by the WAL recovery test"]
fn crash_legacy_fixture_process() {
    let Some(path) = std::env::var_os("COWWORKER_TEST_CRASH_FIXTURE") else {
        return;
    };
    let db = Connection::open(path).unwrap();
    db.execute_batch(include_str!("../src/schema.sql")).unwrap();
    db.execute_batch("PRAGMA journal_mode=WAL; INSERT INTO vacancies VALUES ('v','Role','Company','','Remote','','','','saved','now','now'); INSERT INTO applications VALUES ('a','v','preparing','','','',NULL,'now','now');").unwrap();
    std::process::exit(0);
}

#[test]
fn simultaneous_open_after_legacy_process_exit_preserves_wal_records() {
    for _ in 0..20 {
        let dir = tempfile::tempdir().unwrap();
        let database = dir.path().join("cowworker.db");
        let result = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "crash_legacy_fixture_process", "--ignored"])
            .env("COWWORKER_TEST_CRASH_FIXTURE", &database)
            .stdout(std::process::Stdio::null())
            .status()
            .unwrap();
        assert!(result.success());
        assert!(dir.path().join("cowworker.db-wal").exists());
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(4));
        let mut threads = vec![];
        for _ in 0..4 {
            let path = dir.path().to_path_buf();
            let barrier = barrier.clone();
            threads.push(std::thread::spawn(move || {
                barrier.wait();
                if let Ok(store) = Store::open(path) {
                    assert_eq!(store.workspace().unwrap().vacancies.len(), 1);
                    assert_eq!(store.workspace().unwrap().applications.len(), 1);
                }
            }));
        }
        for thread in threads {
            thread.join().unwrap();
        }
        let store = Store::open(dir.path()).unwrap();
        assert_eq!(store.workspace().unwrap().vacancies.len(), 1);
    }
}

#[test]
fn upgrades_both_schema_one_variants_and_preserves_submitted_bytes() {
    for old in [true, false] {
        let dir = tempfile::tempdir().unwrap();
        let db = Connection::open(dir.path().join("cowworker.db")).unwrap();
        let mut schema = include_str!("../src/schema.sql").to_string();
        if old {
            schema = schema.replace(
                "'resume','cover-letter','note','job-offer','agreement','tax-related','other'",
                "'resume','cover-letter','note'",
            );
        }
        db.execute_batch(&schema).unwrap();
        let doc = uuid::Uuid::new_v4().to_string();
        let version = uuid::Uuid::new_v4().to_string();
        let file = format!("{version}.txt");
        std::fs::create_dir(dir.path().join("documents")).unwrap();
        std::fs::write(
            dir.path().join("documents").join(&file),
            "Точный текст\r\nVersion one",
        )
        .unwrap();
        db.execute(
            "INSERT INTO documents VALUES (?1,'Resume','resume')",
            [&doc],
        )
        .unwrap();
        db.execute(
            "INSERT INTO document_versions VALUES (?1,?2,1,?3,'2026-09-13')",
            [&version, &doc, &file],
        )
        .unwrap();
        db.execute_batch("INSERT INTO vacancies VALUES ('v','Role','Company','','Remote','','','','saved','now','now'); INSERT INTO applications VALUES ('a','v','applied','','','', 'now','now','now'); INSERT INTO application_events VALUES ('e','a','applied','now');").unwrap();
        db.execute(
            "INSERT INTO application_documents VALUES ('a',?1)",
            [&version],
        )
        .unwrap();
        drop(db);
        let mut store = Store::open(dir.path()).unwrap();
        for kind in [
            "resume",
            "cover-letter",
            "note",
            "job-offer",
            "agreement",
            "tax-related",
            "other",
        ] {
            store
                .save_document(DocumentInput {
                    id: None,
                    expected_revision: None,
                    title: kind.into(),
                    kind: kind.into(),
                    content: "test".into(),
                })
                .unwrap();
        }
        let backup_root = tempfile::tempdir().unwrap();
        let backup_path = backup_root.path().join("backup");
        store.backup(&backup_path).unwrap();
        let restored_path = backup_root.path().join("restored");
        backup::restore(&backup_path, &restored_path).unwrap();
        let restored = Store::open(&restored_path).unwrap();
        let data = restored.workspace().unwrap();
        assert_eq!(data.schema_version, 9);
        assert_eq!(
            data.applications[0].document_version_ids,
            vec![version.clone()]
        );
        assert_eq!(data.applications[0].events[0].id, "e");
        assert_eq!(
            std::fs::read(restored_path.join("documents").join(file)).unwrap(),
            "Точный текст\r\nVersion one".as_bytes()
        );
        assert!(backup::restore(&backup_path, &restored_path).is_err());
        std::fs::write(
            backup_path.join("documents").join(format!("{version}.txt")),
            "corrupt",
        )
        .unwrap();
        assert!(backup::verify(&backup_path)
            .unwrap_err()
            .contains("checksum"));
    }
}

#[test]
fn live_wal_snapshot_is_consistent_and_reopen_is_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    store
        .save_profile(Profile {
            name: "WAL only".into(),
            ..Default::default()
        })
        .unwrap();
    let root = tempfile::tempdir().unwrap();
    store.backup(&root.path().join("backup")).unwrap();
    let restored = root.path().join("restored");
    backup::restore(&root.path().join("backup"), &restored).unwrap();
    for _ in 0..2 {
        assert_eq!(
            Store::open(&restored)
                .unwrap()
                .workspace()
                .unwrap()
                .profile
                .name,
            "WAL only"
        );
    }
}

#[test]
fn failed_migration_rolls_back_and_preserves_version() {
    let dir = tempfile::tempdir().unwrap();
    let db = Connection::open(dir.path().join("cowworker.db")).unwrap();
    db.execute_batch(include_str!("../src/schema.sql")).unwrap();
    db.execute_batch("CREATE TABLE documents_new (conflict TEXT);")
        .unwrap();
    assert!(Store::open(dir.path()).is_err());
    assert_eq!(
        db.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        db.query_row("SELECT COUNT(*) FROM documents", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        0
    );
}

#[test]
fn all_work_modes_cross_the_json_contract() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    for mode in ["Remote", "Hybrid", "On-site", "Unspecified"] {
        let input = serde_json::from_value(serde_json::json!({"title":"Role","company":"Company","location":"","workMode":mode,"sourceUrl":"","description":"","notes":"","status":"saved"})).unwrap();
        store.save_vacancy(input).unwrap();
    }
    assert_eq!(store.workspace().unwrap().vacancies.len(), 4);
}
