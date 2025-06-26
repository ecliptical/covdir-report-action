use assert_cmd::{Command, crate_name};
use assert_fs::{TempDir, prelude::*};
use predicates::prelude::*;
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct Action {
    outputs: HashMap<String, serde_yaml_ng::Value>,
}

fn write_input(f: &impl FileWriteStr, root: &Value) -> anyhow::Result<()> {
    let data = serde_json::to_string(root)?;
    f.write_str(&data)?;
    Ok(())
}

#[test]
fn full_options() {
    let tmp_dir = TempDir::new().expect("failed to create temp dir");

    let input = tmp_dir.child("covdir.json");
    write_input(
        &input,
        &json!({
            "name": "",
            "linesCovered": 12,
            "linesMissed": 34,
            "linesTotal": 46,
            "coveragePercent": 12.34,
        }),
    )
    .expect("failed to write input file");

    let summary = tmp_dir.child("summary.md");
    let output = tmp_dir.child("output.txt");
    let results = tmp_dir.child("results.md");

    let mut cmd = Command::cargo_bin(crate_name!()).expect("failed to create cmd");

    let assert = cmd
        .env("GITHUB_OUTPUT", output.path())
        .env("GITHUB_STEP_SUMMARY", summary.path())
        .arg(format!(
            "--file={}",
            input.path().to_str().unwrap_or_default()
        ))
        .arg("--summary=true")
        .arg(format!("--out={}", results.path().to_string_lossy()))
        .assert();
    assert.success();

    let action: Action = serde_yaml_ng::from_slice(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/action.yaml"
    )))
    .expect("failed to read action.yaml");

    for name in action.outputs.keys() {
        output.assert(predicate::str::contains(format!("{name}=")));
    }

    summary.assert(predicate::str::is_empty().not());
    results.assert(predicate::str::is_empty().not());
}

#[test]
fn missing_env() {
    let tmp_dir = TempDir::new().expect("failed to create temp dir");
    let input = tmp_dir.child("covdir.json");
    let mut cmd = Command::cargo_bin(crate_name!()).expect("failed to create cmd");

    let assert = cmd
        .arg(format!(
            "--file={}",
            input.path().to_str().unwrap_or_default()
        ))
        .assert();
    assert.failure();
}
