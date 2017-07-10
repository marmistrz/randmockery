#[cfg(test)]
mod tests {
    #[test]
    fn integration() {
        use std::process::Command;

        let status = Command::new("make").arg("getrandom-test").status().unwrap();
        assert!(status.success());
        let status = Command::new("target/debug/ptrace-rust")
            .arg("./getrandom-test")
            .status()
            .unwrap();
        assert!(status.success());
    }
}