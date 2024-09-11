use std::io::Error;
use std::time::SystemTime;
use tokio::io::{AsyncReadExt, Stdin};

pub trait AsyncLogReader {
    fn read_logs(
        &mut self,
        buf: &mut [u8],
        time: &mut SystemTime,
    ) -> impl std::future::Future<Output = Result<usize, std::io::Error>> + Send;
}

impl AsyncLogReader for Stdin {
    async fn read_logs(&mut self, buf: &mut [u8], time: &mut SystemTime) -> Result<usize, Error> {
        *time = SystemTime::now();
        self.read(buf).await
    }
}
