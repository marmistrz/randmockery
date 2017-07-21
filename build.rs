fn main() {

    use std::process::Command;

    let status = Command::new("make")
        .args(&["-C", "tests", "clean"])
        .status()
        .unwrap();
    assert!(status.success());

    let status = Command::new("make")
        .args(&["-C", "tests"])
        .status()
        .unwrap();
    assert!(status.success());
}
