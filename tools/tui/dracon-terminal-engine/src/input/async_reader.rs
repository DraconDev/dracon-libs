//! Async input reader for use with Tokio.

#[cfg(feature = "async")]
use super::parser::Parser;

#[cfg(feature = "async")]
use super::event::Event;

#[cfg(feature = "async")]
use tokio::sync::mpsc;

#[cfg(feature = "async")]
pub struct AsyncInputReader;

#[cfg(feature = "async")]
impl AsyncInputReader {
    pub fn spawn<F>(mut callback: F) -> tokio::task::JoinHandle<()>
    where
        F: FnMut(Event) + Send + 'static,
    {
        tokio::spawn(async move {
            let mut parser = Parser::new();
            let mut stdin = std::io::stdin();
            let mut buffer = [0u8; 1024];

            loop {
                let mut buf = [0u8; 1024];
                match stdin.read(&mut buf).await {
                    Ok(0) => break,
                    Ok(n) => {
                        for item in buf.iter().take(n) {
                            if let Some(event) = parser.advance(*item) {
                                callback(event);
                            }
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
                        if let Some(evt) = parser.check_timeout() {
                            callback(evt);
                        }
                        continue;
                    }
                    Err(_) => break,
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
            }
        })
    }

    pub fn spawn_with_channel<F>(
        mut callback: F,
    ) -> (tokio::task::JoinHandle<()>, mpsc::Sender<()>)
    where
        F: FnMut(Event) + Send + 'static,
    {
        let (tx, mut rx) = mpsc::channel(1);
        let handle = tokio::spawn(async move {
            let mut parser = Parser::new();
            let mut stdin = std::io::stdin();

            loop {
                tokio::select! {
                    biased;

                    _ = rx.recv() => {
                        break;
                    }
                    result = stdin.read(&mut [0u8; 1024]) => {
                        match result {
                            Ok(0) => break,
                            Ok(n) => {
                                let mut buf = [0u8; 1024];
                                stdin.read(&mut buf).await.ok();
                            }
                            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
                                if let Some(evt) = parser.check_timeout() {
                                    callback(evt);
                                }
                                continue;
                            }
                            Err(_) => break,
                        }
                    }
                }
            }
        });
        (handle, tx)
    }
}