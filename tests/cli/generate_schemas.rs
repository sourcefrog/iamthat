// Copyright 2023 Martin Pool

use assert_fs::TempDir;
use serde_json::Value;

use super::run;

#[test]
fn generate_schemas() {
    let dir = TempDir::new().unwrap();
    run()
        .arg("json-schema")
        .arg("-o")
        .arg(dir.path())
        .assert()
        .success();
    let names = glob::glob(dir.path().join("*.json").to_str().unwrap())
        .unwrap()
        .map(|r| r.unwrap())
        .collect::<Vec<_>>();
    println!("names: {names:?}");
    assert!(names.len() >= 5);

    let user_schema = dir.path().join("user.json");
    // It should at least look like json.
    let _val: Value =
        serde_json::from_reader(std::fs::File::open(user_schema).expect("Open user.json schema"))
            .expect("User schema is json");
}
