
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

#[test]
fn test_version() {
    let mut cmd = Command::cargo_bin("dmarc-cat").unwrap();
    println!("{:?}", cmd);
    let assert = cmd
        .arg("-V")
        .assert();
    assert
        .failure()
        .code(1);
}