#[cfg(test)]
mod tests {
    #[test]
    fn integration() {
        use std::process::Command;

        let status = Command::new("make")
            .args(&["-C", "tests", "getrandom-test"])
            .status()
            .unwrap();
        assert!(status.success());
        let status = Command::new("target/debug/randmockery")
            .arg("tests/getrandom-test")
            .status()
            .unwrap();
        assert!(status.success());
    }
}