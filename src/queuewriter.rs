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
    pub fn new<T: AsyncLogWriter + Send + 'static>(
        mut inner: T,
        limit: usize,
    ) -> (Self, JoinHandle<()>) {
        // TODO: implement channel bounded based on memory size rather than number of elements
        let (tx, mut rx) = mpsc::channel::<LogEvent>(limit);

        let handle = tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                // TODO: log dropped message
                // retries must be handled downstream
                let _ = inner.write_logs(event.timestamp, &event.message).await;
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
            Err(Full(_)) => Ok(()), // TODO: data loss, let the caller decide whether to lose it
            Err(Closed(_)) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Downstream writer is closed",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::writer::MockAsyncLogWriter;
    use mockall::predicate::eq;
    use std::io::Error;
    use std::io::ErrorKind::Other;

    #[tokio::test]
    async fn process_messages() {
        let mut mock = MockAsyncLogWriter::new();

        let time = SystemTime::now();
        mock.expect_write_logs()
            .with(eq(time), eq(b"log1".to_vec()))
            .times(1)
            .returning(|_, _| Ok(()));
        mock.expect_write_logs()
            .with(eq(time), eq(b"log2".to_vec()))
            .times(1)
            .returning(|_, _| Ok(()));

        let (mut writer, handle) = QueueWriter::new(mock, 2);
        writer.write_logs(time, b"log1").await.unwrap();
        writer.write_logs(time, b"log2").await.unwrap();
        drop(writer);
        handle.await.unwrap();
    }

    #[tokio::test]
    async fn drop_message_after_reaching_limit() {
        // TODO
    }

    #[tokio::test]
    async fn retry_message_after_error() {
        let mut mock = MockAsyncLogWriter::new();

        let time = SystemTime::now();
        mock.expect_write_logs()
            .with(eq(time), eq(b"log1".to_vec()))
            .times(1)
            .returning(|_, _| Err(Error::new(Other, "Error")));
        mock.expect_write_logs()
            .with(eq(time), eq(b"log1".to_vec()))
            .times(1)
            .returning(|_, _| Ok(()));
        mock.expect_write_logs()
            .with(eq(time), eq(b"log1".to_vec()))
            .times(0);

        let (mut writer, handle) = QueueWriter::new(mock, 1);
        writer.write_logs(time, b"log1").await.unwrap();
        drop(writer);

        handle.await.unwrap();
    }

    #[tokio::test]
    async fn drop_message_after_max_retries() {
        let mut mock = MockAsyncLogWriter::new();

        let time = SystemTime::now();
        mock.expect_write_logs()
            .with(eq(time), eq(b"log1".to_vec()))
            .times(3)
            .returning(|_, _| Err(Error::new(Other, "Error")));
        mock.expect_write_logs()
            .with(eq(time), eq(b"log1".to_vec()))
            .times(0);

        let (mut writer, handle) = QueueWriter::new(mock, 1);
        writer.write_logs(time, b"log1").await.unwrap();
        drop(writer);

        handle.await.unwrap();
    }
}
