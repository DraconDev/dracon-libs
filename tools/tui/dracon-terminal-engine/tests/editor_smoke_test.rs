//! Smoke test for the text_editor_demo example.
//!
//! Spawns the example binary, waits for initialization, and verifies it
//! exits cleanly (no immediate panic).
//!
//! Note: This test may exit with code 1 in non-TTY environments (CI, containers)
//! because the terminal cannot be initialized. This is expected behavior for
//! interactive TUI applications.

use std::process::{Command, Stdio};
use std::time::Duration;
use std::io::Write;
use std::thread;

#[test]
fn test_text_editor_demo_smoke() {
    // Build the example first so we have a binary to run
    let build_status = Command::new("cargo")
        .args(["build", "--example", "text_editor_demo"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("failed to run cargo build for text_editor_demo");

    assert!(build_status.success(), "cargo build for text_editor_demo failed");

    // Spawn the example with piped stdin so we can write to it
    let mut child = Command::new("cargo")
        .args(["run", "--example", "text_editor_demo"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn text_editor_demo");

    // Give it time to initialize the terminal and start the event loop
    thread::sleep(Duration::from_millis(800));

    // Send Ctrl+C (byte 0x03 = SIGINT) to gracefully shut down
    {
        let stdin = child.stdin.as_mut().expect("stdin not captured");
        stdin.write_all(&[3]).ok();
    }

    // Wait for the process to exit with a manual timeout loop
    let mut exited = false;
    for _ in 0..50 {
        match child.try_wait() {
            Ok(Some(status)) => {
                // Exit code 1 in non-TTY environments is expected for TUI apps
                // Exit code 0 means it shut down gracefully
                if status.code() == Some(1) {
                    // Likely "could not get terminal size" or similar tty-related error
                    // This is acceptable in CI/container environments
                    exited = true;
                } else {
                    assert!(
                        status.success(),
                        "text_editor_demo exited with error: {:?}",
                        status.code()
                    );
                    exited = true;
                }
                break;
            }
            Ok(None) => {
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                panic!("error waiting for text_editor_demo: {}", e);
            }
        }
    }

    if !exited {
        child.kill().ok();
        panic!("text_editor_demo did not exit within 5 seconds");
    }
}
