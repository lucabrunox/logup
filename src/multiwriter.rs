use crate::writer::AsyncLogWriter;
use async_trait::async_trait;
use std::time::SystemTime;

pub struct MultiWriter<T>
where
    T: AsyncLogWriter,
{
    writers: Vec<T>,
}

impl<T: AsyncLogWriter> MultiWriter<T> {
    pub fn new(writers: Vec<T>) -> MultiWriter<T> {
        MultiWriter { writers }
    }
}

#[async_trait]
impl<T: AsyncLogWriter + Send> AsyncLogWriter for MultiWriter<T> {
    async fn write_logs(&mut self, time: SystemTime, buf: &[u8]) -> std::io::Result<()> {
        for writer in self.writers.iter_mut() {
            writer.write_logs(time, buf).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::writer::MockAsyncLogWriter;
    use mockall::predicate::*;
    use std::ops::Add;
    use std::time::Duration;

    #[tokio::test]
    async fn test_write_to_multiple_writers() {
        let mut mock_writer1 = MockAsyncLogWriter::new();
        let mut mock_writer2 = MockAsyncLogWriter::new();

        let time1 = SystemTime::now();
        let buf1: &[u8] = b"test1";
        let time2 = time1.add(Duration::new(100, 0));
        let buf2: &[u8] = b"test2";
        mock_writer1
            .expect_write_logs()
            .with(eq(time1), eq(buf1))
            .returning(|_, _| Ok(()));
        mock_writer1
            .expect_write_logs()
            .with(eq(time2), eq(buf2))
            .returning(|_, _| Ok(()));
        mock_writer2
            .expect_write_logs()
            .with(eq(time1), eq(buf1))
            .returning(|_, _| Ok(()));
        mock_writer2
            .expect_write_logs()
            .with(eq(time2), eq(buf2))
            .returning(|_, _| Ok(()));

        let mut multi_writer = MultiWriter::new(vec![mock_writer1, mock_writer2]);

        let result = multi_writer.write_logs(time1, buf1).await;
        assert!(result.is_ok());
        let result = multi_writer.write_logs(time2, buf2).await;
        assert!(result.is_ok());
    }
}
