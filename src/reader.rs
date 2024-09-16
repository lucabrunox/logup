use async_trait::async_trait;
use std::io::Error;
use std::time::SystemTime;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, Stdin};

#[async_trait]
pub trait AsyncLogReader {
    async fn read_logs(
        &mut self,
        buf: &mut [u8],
        time: &mut SystemTime,
    ) -> Result<usize, std::io::Error>;
}

#[async_trait]
impl AsyncLogReader for Stdin {
    async fn read_logs(&mut self, buf: &mut [u8], time: &mut SystemTime) -> Result<usize, Error> {
        *time = SystemTime::now();
        self.read(buf).await
    }
}

#[async_trait]
impl AsyncLogReader for File {
    async fn read_logs(&mut self, buf: &mut [u8], time: &mut SystemTime) -> Result<usize, Error> {
        *time = SystemTime::now();
        self.read(buf).await
    }
}

#[async_trait]
impl<T: AsyncLogReader + Send + ?Sized> AsyncLogReader for Box<T> {
    async fn read_logs(&mut self, buf: &mut [u8], time: &mut SystemTime) -> Result<usize, Error> {
        (**self).read_logs(buf, time).await
    }
}
