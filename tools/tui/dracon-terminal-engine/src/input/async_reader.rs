//! Async input reader for use with Tokio.
//!
//! Enable with `features = ["async"]` in Cargo.toml.

#[cfg(feature = "async")]
use tokio::sync::mpsc;

#[cfg(feature = "async")]
use tokio::time::{sleep, Duration};

#[cfg(feature = "async")]
pub struct AsyncInputReader;

#[cfg(feature = "async")]
impl AsyncInputReader {
    pub fn spawn<F>(mut callback: F) -> tokio::task::JoinHandle<()>
    where
        F: FnMut(crate::input::event::Event) + Send + 'static,
    {
        tokio::spawn(async move {
            let mut parser = crate::input::parser::Parser::new();
            let mut buffer = [0u8; 1024];

            loop {
                let n = tokio::task::spawn_blocking(move || {
                    let mut stdin = std::io::stdin();
                    std::io::Read::read(&mut stdin, &mut buffer)
                }).await;

                match n {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        for &item in buffer.iter().take(n) {
                            if let Some(event) = parser.advance(item) {
                                callback(event);
                            }
                        }
                    }
                }

                sleep(Duration::from_millis(20)).await;

                if let Some(evt) = parser.check_timeout() {
                    callback(evt);
                }
            }
        })
    }

    pub fn spawn_with_shutdown<F>(
        mut callback: F,
    ) -> (tokio::task::JoinHandle<()>, ShutdownGuard)
    where
        F: FnMut(crate::input::event::Event) + Send + 'static,
    {
        let (tx, rx) = mpsc::channel(1);
        let handle = tokio::spawn(async move {
            let mut parser = crate::input::parser::Parser::new();
            let mut buffer = [0u8; 1024];
            let mut rx = rx;

            loop {
                let read_result = tokio::task::spawn_blocking(move || {
                    let mut stdin = std::io::stdin();
                    std::io::Read::read(&mut stdin, &mut buffer)
                }).await;

                let should_break = match read_result {
                    Ok(0) | Err(_) => true,
                    Ok(n) => {
                        for &item in buffer.iter().take(n) {
                            if let Some(event) = parser.advance(item) {
                                callback(event);
                            }
                        }
                        false
                    }
                };

                if should_break {
                    break;
                }

                tokio::select! {
                    _ = rx.recv() => {
                        break;
                    }
                    _ = sleep(Duration::from_millis(20)) => {
                        if let Some(evt) = parser.check_timeout() {
                            callback(evt);
                        }
                    }
                }
            }
        });
        (handle, ShutdownGuard { tx })
    }
}

#[cfg(feature = "async")]
pub struct ShutdownGuard {
    tx: mpsc::Sender<()>,
}

#[cfg(feature = "async")]
impl ShutdownGuard {
    pub fn shutdown(self) {
        drop(self.tx);
    }
}