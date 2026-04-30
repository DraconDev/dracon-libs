//! Smoke test for the text_editor_demo example.
//!
//! Spawns the example binary, waits for it to initialize, sends Ctrl+C,
//! and verifies the process exits cleanly (exit code 0, no panic).

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
        // try_wait returns None if the child hasn't exited yet
        match child.try_wait() {
            Ok(Some(status)) => {
                assert!(
                    status.success(),
                    "text_editor_demo exited with error signal: {:?}",
                    status.code()
                );
                exited = true;
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
