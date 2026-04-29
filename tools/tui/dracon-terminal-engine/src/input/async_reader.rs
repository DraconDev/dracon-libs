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
                let result = tokio::task::spawn_blocking(move || {
                    let mut stdin = std::io::stdin();
                    stdin.read(&mut buffer)
                }).await;

                let n = match result {
                    Ok(n) => n,
                    Err(_) => break,
                };

                if n == 0 {
                    break;
                }

                for &item in buffer.iter().take(n) {
                    if let Some(event) = parser.advance(item) {
                        callback(event);
                    }
                }

                sleep(Duration::from_millis(20)).await;

                if let Some(evt) = parser.check_timeout() {
                    callback(evt);
                }
            }
        })
    }

    pub fn spawn_with_channel<F>(
        mut callback: F,
    ) -> (tokio::task::JoinHandle<()>, mpsc::Sender<()>)
    where
        F: FnMut(crate::input::event::Event) + Send + 'static,
    {
        let (tx, mut rx) = mpsc::channel(1);
        let handle = tokio::spawn(async move {
            let mut parser = crate::input::parser::Parser::new();
            let mut buffer = [0u8; 1024];

            loop {
                let n = tokio::task::spawn_blocking(move || {
                    let mut stdin = std::io::stdin();
                    stdin.read(&mut buffer)
                }).await;

                let n = match n {
                    Ok(n) => n,
                    Err(_) => break,
                };

                tokio::select! {
                    biased;

                    _ = rx.recv() => {
                        break;
                    }
                    _ = async {} => {
                        if n == 0 {
                            break;
                        }
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
        });
        (handle, tx)
    }
}