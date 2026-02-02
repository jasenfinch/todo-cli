use assert_cmd::cargo;
use assert_cmd::Command;
use tempfile::TempDir;

#[test]
fn test_todo() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let mut cmd = Command::new(cargo::cargo_bin!());
    cmd.arg("-p").arg(temp_path).arg("list").assert().success();

    let mut cmd = Command::new(cargo::cargo_bin!());
    cmd.arg("-p")
        .arg(temp_path)
        .args(["add", "test", "-d", "a test task", "-t", "work,project"])
        .assert()
        .success();

    let mut cmd = Command::new(cargo::cargo_bin!());
    cmd.arg("-p").arg(temp_path).arg("list").assert().success();

    let mut cmd = Command::new(cargo::cargo_bin!());
    cmd.arg("-p").arg(temp_path).arg("next").assert().success();

    let mut cmd = Command::new(cargo::cargo_bin!());
    cmd.arg("-p").arg(temp_path).arg("clear").assert().success();
}
