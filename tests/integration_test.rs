//! Integration tests for ephemeral file share

use std::process::Command;

#[test]
fn test_binary_compiles() {
    let output = Command::new("cargo")
        .args(["build", "--release"])
        .output()
        .expect("Failed to execute cargo build");

    assert!(output.status.success(), "Build failed: {}", String::from_utf8_lossy(&output.stderr));
}

#[test]
fn test_unit_tests_pass() {
    let output = Command::new("cargo")
        .args(["test"])
        .output()
        .expect("Failed to execute cargo test");

    assert!(output.status.success(), "Tests failed: {}", String::from_utf8_lossy(&output.stderr));
}
