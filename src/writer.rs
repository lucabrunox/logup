use std::time::SystemTime;
use async_trait::async_trait;
use tokio::io::AsyncWriteExt;

#[cfg(test)]
use mockall::{automock, predicate::*};

#[cfg_attr(test, automock)]
#[async_trait]
pub trait AsyncLogWriter {
    async fn write_logs(&mut self, time: SystemTime, buf: &[u8]) -> std::io::Result<()>;
}

#[async_trait]
impl AsyncLogWriter for tokio::io::Stdout {
    async fn write_logs(&mut self, _time: SystemTime, buf: &[u8]) -> std::io::Result<()> {
        self.write_all(buf).await
    }
}

#[async_trait]
impl<T: AsyncLogWriter + Send + ?Sized> AsyncLogWriter for Box<T> {
    async fn write_logs(&mut self, time: SystemTime, buf: &[u8]) -> std::io::Result<()> {
        (**self).write_logs(time, buf).await
    }
}
