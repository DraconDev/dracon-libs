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

            loop {
                let n = tokio::task::block_in_place(|| {
                    let mut stdin = std::io::stdin();
                    let mut buf = [0u8; 1024];
                    stdin.read(&mut buf)
                });

                match n {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        let mut buf = [0u8; 1024];
                        let read_n = tokio::task::block_in_place(|| {
                            let mut stdin = std::io::stdin();
                            stdin.read(&mut buf)
                        });
                        match read_n {
                            Ok(0) | Err(_) => break,
                            Ok(read_n) => {
                                for &item in buf.iter().take(read_n) {
                                    if let Some(event) = parser.advance(item) {
                                        callback(event);
                                    }
                                }
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
            let mut rx = rx;

            loop {
                let n = tokio::task::block_in_place(|| {
                    let mut stdin = std::io::stdin();
                    let mut buf = [0u8; 1024];
                    stdin.read(&mut buf)
                });

                match n {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        let mut buf = [0u8; 1024];
                        let read_n = tokio::task::block_in_place(|| {
                            let mut stdin = std::io::stdin();
                            stdin.read(&mut buf)
                        });
                        match read_n {
                            Ok(0) | Err(_) => break,
                            Ok(read_n) => {
                                for &item in buf.iter().take(read_n) {
                                    if let Some(event) = parser.advance(item) {
                                        callback(event);
                                    }
                                }
                            }
                        }
                    }
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