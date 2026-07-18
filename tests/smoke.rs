use assert_cmd::Command;
use predicates::prelude::*;
use predicates::str::contains;

#[test]
fn binary_prints_usage_without_args() {
    Command::cargo_bin("scope")
        .unwrap()
        .assert()
        .failure()
        .stderr(contains("Usage").and(contains("scope")));
}

#[test]
fn help_flag_mentions_interval_option() {
    Command::cargo_bin("scope")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("--interval"));
}
