//! Async input reader for use with Tokio.

#[cfg(feature = "async")]
use std::os::fd::AsFd;

#[cfg(feature = "async")]
use tokio::io::AsyncReadExt;

#[cfg(feature = "async")]
use super::parser::Parser;

#[cfg(feature = "async")]
use super::event::Event;

#[cfg(feature = "async")]
use crate::backend::tty;

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
            let mut tick_interval = tokio::time::interval(tokio::time::Duration::from_millis(20));

            loop {
                tokio::select! {
                    _ = tick_interval.tick() => {
                        let stdin_fd = stdin.as_fd();
                        match tty::poll_input_async(stdin_fd, 0).await {
                            Ok(true) => {
                                match stdin.read(&mut buffer).await {
                                    Ok(0) => break,
                                    Ok(n) => {
                                        for item in buffer.iter().take(n) {
                                            if let Some(event) = parser.advance(*item) {
                                                callback(event);
                                            }
                                        }
                                    }
                                    Err(_) => break,
                                }
                            }
                            Ok(false) => {
                                if let Some(evt) = parser.check_timeout() {
                                    callback(evt);
                                }
                            }
                            Err(_) => break,
                        }
                    }
                }
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
            let mut buffer = [0u8; 1024];
            let mut tick_interval = tokio::time::interval(tokio::time::Duration::from_millis(20));

            loop {
                tokio::select! {
                    _ = tick_interval.tick() => {
                        let stdin_fd = stdin.as_fd();
                        match tty::poll_input_async(stdin_fd, 0).await {
                            Ok(true) => {
                                match stdin.read(&mut buffer).await {
                                    Ok(0) => break,
                                    Ok(n) => {
                                        for item in buffer.iter().take(n) {
                                            if let Some(event) = parser.advance(*item) {
                                                callback(event);
                                            }
                                        }
                                    }
                                    Err(_) => break,
                                }
                            }
                            Ok(false) => {
                                if let Some(evt) = parser.check_timeout() {
                                    callback(evt);
                                }
                            }
                            Err(_) => break,
                        }
                    }
                    _ = rx.recv() => {
                        break;
                    }
                }
            }
        });
        (handle, tx)
    }
}

#[cfg(feature = "async")]
pub mod blocking {
    use super::*;
    use std::thread;

    pub fn spawn_blocking<F>(mut callback: F) -> thread::JoinHandle<()>
    where
        F: FnMut(Event) + Send + 'static,
    {
        thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_time()
                .build()
                .expect("Failed to create tokio runtime");
            rt.block_on(async {
                let mut parser = Parser::new();
                let mut stdin = std::io::stdin();
                let mut buffer = [0u8; 1024];

                loop {
                    let stdin_fd = stdin.as_fd();
                    match tty::poll_input(stdin_fd, 20) {
                        Ok(true) => {
                            match stdin.read(&mut buffer) {
                                Ok(0) => break,
                                Ok(n) => {
                                    for item in buffer.iter().take(n) {
                                        if let Some(event) = parser.advance(*item) {
                                            callback(event);
                                        }
                                    }
                                }
                                Err(_) => break,
                            }
                        }
                        Ok(false) => {
                            if let Some(evt) = parser.check_timeout() {
                                callback(evt);
                            }
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                        Err(_) => break,
                    }
                }
            });
        })
    }
}