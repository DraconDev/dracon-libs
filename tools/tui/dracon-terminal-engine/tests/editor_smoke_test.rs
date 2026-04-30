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
    let status = Command::new("cargo")
        .args(["build", "--example", "text_editor_demo"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    assert!(status.is_ok(), "failed to build text_editor_demo example");
    assert!(status.unwrap().success(), "build of text_editor_demo failed");

    // Spawn the example — it will open with "Start typing..." placeholder content
    let mut child = Command::new("cargo")
        .args(["run", "--example", "text_editor_demo"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn text_editor_demo");

    // Give it time to initialize the terminal and start the event loop
    thread::sleep(Duration::from_millis(500));

    // Send Ctrl+C to gracefully shut down (SIGINT)
    // In raw terminal mode, Ctrl+C sends SIGINT to the process group
    {
        let stdin = child.stdin.as_mut().expect("stdin not captured");
        // Ctrl+C = byte 0x03 in the stream
        stdin.write_all(&[3]).ok();
    }

    // Wait for the process to exit (with timeout)
    let result = child.wait_with_timeout(Duration::from_secs(5));

    match result {
        Ok(Some(status)) => {
            assert!(status.success(), "text_editor_demo exited with error: {:?}", status);
        }
        Ok(None) => {
            // Timed out — force kill
            child.kill().ok();
            panic!("text_editor_demo did not exit within 5 seconds");
        }
        Err(e) => {
            panic!("error waiting for text_editor_demo: {}", e);
        }
    }
}
