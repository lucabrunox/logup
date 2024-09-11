use crate::writer::AsyncLogWriter;
use async_trait::async_trait;
use std::time::SystemTime;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TrySendError::{Closed, Full};
use tokio::task::JoinHandle;

struct LogEvent {
    timestamp: SystemTime,
    message: Vec<u8>,
}

pub struct QueueWriter {
    tx: mpsc::Sender<LogEvent>,
}

impl QueueWriter {
    pub fn new<T: AsyncLogWriter + Send + 'static>(mut inner: T) -> (Self, JoinHandle<()>) {
        // TODO: implement channel bounded based on memory size rather than number of elements
        let (tx, mut rx) = mpsc::channel::<LogEvent>(100);

        let handle = tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                // TODO: make it configurable
                let mut retries = 100;
                while inner
                    .write_logs(event.timestamp, &event.message)
                    .await
                    .is_err()
                {
                    // TODO: log
                    if retries <= 0 {
                        break;
                    }
                    retries -= 1;
                }
            }
        });
        (Self { tx }, handle)
    }
}

#[async_trait]
impl AsyncLogWriter for QueueWriter {
    async fn write_logs(&mut self, time: SystemTime, buf: &[u8]) -> std::io::Result<()> {
        match self.tx.try_send(LogEvent {
            timestamp: time,
            message: buf.into(),
        }) {
            Ok(_) => Ok(()),
            Err(Full(_)) => Ok(()), // TODO: let the caller decide whether to drop
            Err(Closed(_)) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Downstream writer is closed",
            )),
        }
    }
}
