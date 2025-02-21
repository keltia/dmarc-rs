use assert_cmd::prelude::*;
use std::process::Command;

#[test]
fn test_empty_args() {
    let mut cmd = Command::new(std::env::var("CARGO_BIN_EXE_dmarc-rs").unwrap());
    cmd.assert().failure();
}

#[test]
fn test_help() {
    let mut cmd = Command::new(std::env::var("CARGO_BIN_EXE_dmarc-rs").unwrap());
    cmd.arg("-h").assert().success();
}

#[test]
fn test_version() {
    let mut cmd = Command::new(std::env::var("CARGO_BIN_EXE_dmarc-rs").unwrap());

    cmd.arg("-V").assert().success();
}

#[test]
fn test_invalid_type_nok() {
    let mut cmd = Command::new(std::env::var("CARGO_BIN_EXE_dmarc-rs").unwrap());
    cmd.arg("-t").arg("blah").assert().failure();
}

#[test]
fn test_invalid_type_ok() {
    let mut cmd = Command::new(std::env::var("CARGO_BIN_EXE_dmarc-rs").unwrap());
    cmd.arg("-t").arg("csv").assert().failure();
}
