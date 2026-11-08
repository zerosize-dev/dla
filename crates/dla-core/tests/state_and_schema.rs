mod common;

use common::*;
use dla_core::{
    Error, RunState, RunStatus, SCHEMA_VERSION, Stage, schema,
    state::{read_current, write_current},
    version::{Migration, SchemaErrorKind, UpgradeError, upgrade_with},
};
use serde_json::{Value, json};

#[test]
fn run_state_init_shape() {
    let s = run_state();
    assert_eq!(s.schema_version, SCHEMA_VERSION);
    assert_eq!((s.stage, s.status), (Stage::Intake, RunStatus::Active));
    let (stamp, suffix) = s.run_id.rsplit_once('-').unwrap();
    assert_eq!(stamp.len(), 15);
    assert_eq!(suffix.len(), 4);
    assert!(suffix.bytes().all(|b| b.is_ascii_hexdigit()));
    assert_eq!(s.redaction_salt.len(), 32);
    assert_eq!(s.salt_bytes().len(), 16);
    assert_eq!(
        s.budget.deadline - s.budget.started_at,
        chrono::TimeDelta::seconds(3600)
    );
    let unknown = RunState::init("/p", None, 60, None, dla_core::state::now_utc()).unwrap();
    assert_eq!(unknown.session_id, "unknown");
    assert_ne!(s.redaction_salt, unknown.redaction_salt);
}

#[test]
fn state_save_load_is_atomic_and_round_trips() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = run_state();
    s.save(dir.path()).unwrap();
    s.stage = Stage::Probe;
    s.save(dir.path()).unwrap();
    assert_eq!(RunState::load(dir.path()).unwrap().stage, Stage::Probe);
    let names: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(names, ["state.json"], "no temp files left behind");
    assert!(
        std::fs::read_to_string(dir.path().join("state.json"))
            .unwrap()
            .ends_with('\n')
    );
}

#[test]
fn state_schema_versions() {
    let dir = tempfile::tempdir().unwrap();
    let s = run_state();
    s.save(dir.path()).unwrap();
    let path = dir.path().join("state.json");
    let base: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let with = |v: u32| {
        let mut d = base.clone();
        d["schema_version"] = json!(v);
        std::fs::write(&path, d.to_string()).unwrap();
        RunState::load(dir.path())
    };
    let e = with(2).unwrap_err();
    assert!(matches!(e, Error::Schema(_)) && e.exit_code() == 3);
    assert!(e.to_string().contains("newer dla"));
    let e = with(0).unwrap_err();
    assert!(matches!(e, Error::Schema(_)));
    assert!(e.to_string().contains("cannot migrate schema 0"));
    assert!(with(1).is_ok());
    std::fs::write(&path, "{").unwrap();
    assert_eq!(RunState::load(dir.path()).unwrap_err().exit_code(), 1);
}

#[test]
fn current_pointer_round_trips() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(read_current(dir.path()).unwrap(), None);
    std::fs::create_dir(dir.path().join(".dla")).unwrap();
    write_current(dir.path(), "20260101-000000-abcd").unwrap();
    assert_eq!(
        read_current(dir.path()).unwrap().as_deref(),
        Some("20260101-000000-abcd")
    );
}

fn bump_name(mut v: Value) -> Result<Value, String> {
    let old = v["name"].take();
    v["title"] = old;
    Ok(v)
}

#[test]
fn migration_mechanism_chains_steps() {
    let table = [
        Migration {
            from: 1,
            apply: bump_name,
        },
        Migration { from: 2, apply: Ok },
    ];
    let out = upgrade_with(json!({"schema_version": 1, "name": "x"}), &table, 3, "doc").unwrap();
    assert_eq!(
        out,
        json!({"schema_version": 3, "title": "x", "name": null})
    );
    // Already current: untouched.
    let doc = json!({"schema_version": 3});
    assert_eq!(upgrade_with(doc.clone(), &table, 3, "doc").unwrap(), doc);
}

#[test]
fn migration_gaps_failures_and_newer() {
    let table = [Migration { from: 1, apply: Ok }];
    let Err(UpgradeError::Schema(e)) = upgrade_with(json!({"schema_version": 0}), &table, 2, "doc")
    else {
        panic!()
    };
    assert_eq!((e.found, e.kind), (0, SchemaErrorKind::CannotMigrate));
    let Err(UpgradeError::Schema(e)) = upgrade_with(json!({"schema_version": 1}), &table, 3, "doc")
    else {
        panic!()
    };
    assert_eq!(e.kind, SchemaErrorKind::CannotMigrate);
    let failing = [Migration {
        from: 1,
        apply: |_| Err("boom".into()),
    }];
    let Err(UpgradeError::Schema(e)) =
        upgrade_with(json!({"schema_version": 1}), &failing, 2, "doc")
    else {
        panic!()
    };
    assert!(matches!(e.kind, SchemaErrorKind::MigrationFailed(ref m) if m == "boom"));
    let Err(UpgradeError::Schema(e)) = upgrade_with(json!({"schema_version": 5}), &table, 2, "doc")
    else {
        panic!()
    };
    assert_eq!(e.kind, SchemaErrorKind::Newer);
    for bad in [
        json!({}),
        json!({"schema_version": "1"}),
        json!({"schema_version": -1}),
        json!([]),
    ] {
        assert!(matches!(
            upgrade_with(bad, &table, 2, "doc"),
            Err(UpgradeError::Malformed(_))
        ));
    }
}

#[test]
fn schema_export_is_complete_and_deterministic() {
    let all = schema::all();
    let names: Vec<_> = all.iter().map(|(n, _)| *n).collect();
    assert_eq!(
        names,
        [
            "evidence-record",
            "finding",
            "ledger-line",
            "run-state",
            "coverage-manifest",
            "checklist",
            "detectors",
            "probe-report",
            "static-hit",
            "census-entry",
            "workload",
            "workload-report",
            "series",
            "trend",
            "harness-manifest",
            "harness-dump"
        ]
    );
    let first: Vec<String> = all.iter().map(|(_, s)| schema::render(s)).collect();
    let second: Vec<String> = schema::all()
        .iter()
        .map(|(_, s)| schema::render(s))
        .collect();
    assert_eq!(first, second);
    for text in &first {
        assert!(text.ends_with("}\n"));
        let v: Value = serde_json::from_str(text).unwrap();
        assert!(v.get("$schema").is_some() || v.get("type").is_some());
    }
    let finding = &first[1];
    assert!(finding.contains("suspected_static_only") && finding.contains("registration-table"));
}

#[test]
fn result_size_constants() {
    assert_eq!(dla_core::MAX_RESULT_BYTES, 2048);
    assert_eq!(dla_core::MAX_OUTPUT_BYTES, 2048);
    assert_eq!(dla_core::SCHEMA_VERSION, 1);
    assert_eq!(dla_core::LEDGER_SCHEMA_VERSION, 2);
}

#[test]
fn evidence_kind_classes() {
    use dla_core::{EvidenceKind as K, evidence::EvidenceClass as C};
    let cases = [
        (K::GrowthSeries, C::Growth),
        (K::UnitCountSeries, C::Growth),
        (K::RetentionPath, C::Retention),
        (K::HistogramDiff, C::Retention),
        (K::AllocStack, C::Allocation),
        (K::Intervention, C::Intervention),
        (K::Discriminator, C::Discriminator),
        (K::StaticHit, C::Static),
        (K::CensusHolder, C::Static),
        (K::RunConfig, C::Context),
        (K::Probe, C::Context),
        (K::Workload, C::Context),
        (K::BlockedStackProfile, C::Context),
        (K::UnreachableAtExit, C::Context),
        (K::ShouldBeDead, C::Context),
        (K::NonHeap, C::Context),
        (K::Residual, C::Context),
        (K::Artifact, C::Context),
    ];
    assert_eq!(cases.len(), K::ALL.len());
    for (k, c) in cases {
        assert_eq!(k.class(), c, "{k}");
    }
    assert_eq!("retention-path".parse::<K>().unwrap(), K::RetentionPath);
    assert!("nope".parse::<K>().is_err());
}
