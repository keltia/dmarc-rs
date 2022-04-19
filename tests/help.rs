
use assert_cmd::Command;

#[test]
fn test_help() {
    let mut cmd = Command::cargo_bin("dmarc-cat").unwrap();
    let assert = cmd
        .arg("-h")
        .assert();
    assert
        .success();
}
