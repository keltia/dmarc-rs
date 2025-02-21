use assert_cmd::prelude::*;
use std::process::Command;

#[test]
fn test_unknown_file() {
    let mut cmd = Command::new(std::env::var("CARGO_BIN_EXE_dmarc-rs").unwrap());
    cmd.arg("/nonexistent").assert().failure();
}
