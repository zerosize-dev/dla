//! Which dependency variables a harness rerun takes from the last run of the harness.

use std::fs;

use dla_core::harness::{DEPENDENCY_ENV, dependency_env_to_reuse_with};
use serde_json::json;

fn record(dir: &std::path::Path, id: &str, first: &str, env: serde_json::Value) {
    let h = dir.join("harness").join(id);
    fs::create_dir_all(&h).unwrap();
    fs::write(
        h.join(format!("run-{first}.json")),
        json!({"series": [first], "dependency_env": env}).to_string(),
    )
    .unwrap();
}

#[test]
fn only_listed_unset_variables_of_the_latest_run_come_back() {
    let dir = tempfile::tempdir().unwrap();
    record(dir.path(), "H1", "S1", json!({"PYTHONPATH": "/old"}));
    // A record can name anything; only the fixed list is read back.
    record(
        dir.path(),
        "H1",
        "S2",
        json!({"PYTHONPATH": "/new", "NODE_PATH": "/n", "PATH": "/bin", "API_TOKEN": "t"}),
    );
    let unset = |_: &str| false;
    assert_eq!(
        dependency_env_to_reuse_with(dir.path(), "H1", unset),
        [
            ("PYTHONPATH".to_owned(), "/new".to_owned()),
            ("NODE_PATH".to_owned(), "/n".to_owned())
        ]
    );
    // A variable that is set here wins, however it is set.
    assert_eq!(
        dependency_env_to_reuse_with(dir.path(), "H1", |k| k == "PYTHONPATH"),
        [("NODE_PATH".to_owned(), "/n".to_owned())]
    );
    // A harness that never ran, and one whose record predates the field, give nothing.
    assert!(dependency_env_to_reuse_with(dir.path(), "H9", unset).is_empty());
    let h = dir.path().join("harness/H2");
    fs::create_dir_all(&h).unwrap();
    fs::write(h.join("run-S3.json"), json!({"series": ["S3"]}).to_string()).unwrap();
    assert!(dependency_env_to_reuse_with(dir.path(), "H2", unset).is_empty());
}

#[test]
fn an_edited_clone_takes_the_variables_of_the_harness_it_was_cloned_from() {
    let dir = tempfile::tempdir().unwrap();
    record(dir.path(), "H1", "S1", json!({"CLASSPATH": "/jars"}));
    let clone = dir.path().join("harness/H2");
    fs::create_dir_all(&clone).unwrap();
    fs::write(clone.join(dla_core::harness::CLONE_MARKER), "H1").unwrap();
    assert_eq!(
        dependency_env_to_reuse_with(dir.path(), "H2", |_| false),
        [("CLASSPATH".to_owned(), "/jars".to_owned())]
    );
}

#[test]
fn the_list_is_the_fixed_one() {
    assert_eq!(
        DEPENDENCY_ENV,
        [
            "PYTHONPATH",
            "NODE_PATH",
            "CLASSPATH",
            "GOFLAGS",
            "GOWORK",
            "LD_LIBRARY_PATH",
            "DYLD_LIBRARY_PATH",
            "VIRTUAL_ENV"
        ]
    );
}
