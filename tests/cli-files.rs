use assert_cmd::prelude::*;
use std::process::Command;

#[test]
fn test_unknown_file() {
    let mut cmd = Command::new(std::env::var("CARGO_BIN_EXE_dmarc-cat").unwrap());
    cmd.arg("/nonexistent").assert().failure();
}
