use assert_cmd::Command;

const BIN: &str = "dmarc-cat";

#[test]
fn test_empty_args() {
    let mut cmd = Command::cargo_bin(BIN).unwrap();
    cmd.assert().failure();
}

#[test]
fn test_help() {
    let mut cmd = Command::cargo_bin(BIN).unwrap();
    cmd.arg("-h").assert().success();
}

#[test]
fn test_version() {
    let mut cmd = Command::cargo_bin(BIN).unwrap();

    cmd.arg("-V").assert().success();
}

#[test]
fn test_invalid_type_nok() {
    let mut cmd = Command::cargo_bin(BIN).unwrap();
    cmd.arg("-t").arg("blah").assert().failure();
}

#[test]
fn test_invalid_type_ok() {
    let mut cmd = Command::cargo_bin(BIN).unwrap();
    cmd.arg("-t").arg("txt").assert().success();
}
