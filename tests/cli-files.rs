use assert_cmd::Command;

const BIN: &str = "dmarc-cat";

#[test]
fn test_unknown_file() {
    let mut cmd = Command::cargo_bin(BIN).unwrap();
    cmd.arg("/nonexistent").assert().failure();
}
