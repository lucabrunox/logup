use crate::writer::AsyncLogWriter;
use async_trait::async_trait;
use mem::take;
use std::cmp::min;
use std::mem;
use std::time::SystemTime;

pub struct LinesWriter<T: AsyncLogWriter> {
    inner: T,
    buf: Vec<u8>,
    max_line_size: usize,
}

impl<T: AsyncLogWriter> LinesWriter<T> {
    pub fn new(inner: T, max_line_size: usize) -> Self {
        Self {
            inner,
            buf: Vec::new(),
            max_line_size,
        }
    }
}

#[async_trait]
impl<T: AsyncLogWriter + Send> AsyncLogWriter for LinesWriter<T> {
    async fn write_logs(&mut self, time: SystemTime, buf: &[u8]) -> std::io::Result<()> {
        let mut buf = buf;
        while !buf.is_empty() {
            if let Some(pos) = buf.iter().position(|&b| b == b'\n') {
                let len = min(self.buf.len() + pos + 1, self.max_line_size) - self.buf.len();
                let line = &buf[..len];
                if !self.buf.is_empty() {
                    let mut newbuf = take(&mut self.buf);
                    newbuf.extend_from_slice(line); // concatenate to previous buffer
                    self.buf.clear(); // clear before the next line could fail
                    self.inner.write_logs(time, newbuf.as_slice()).await?;
                } else {
                    self.inner.write_logs(time, line).await?;
                }
                buf = &buf[len..];
            } else {
                // no newline found, buffer
                let len = min(self.buf.len() + buf.len(), self.max_line_size) - self.buf.len();
                buf = &buf[..len];
                self.buf.extend_from_slice(buf);
                buf = &buf[len..];
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::writer::{AsyncLogWriter, MockAsyncLogWriter};
    use crate::writer_lines::LinesWriter;
    use mockall::predicate::eq;
    use std::time::SystemTime;

    #[tokio::test]
    async fn split_lines1() {
        let mut mock = MockAsyncLogWriter::new();
        let time = SystemTime::now();

        mock.expect_write_logs()
            .with(eq(time), eq(b"line1\n".to_vec()))
            .times(1)
            .returning(|_, _| Ok(()));
        mock.expect_write_logs()
            .with(eq(time), eq(b"line2\n".to_vec()))
            .times(1)
            .returning(|_, _| Ok(()));
        mock.expect_write_logs()
            .with(eq(time), eq(b"line3extra\n".to_vec()))
            .times(1)
            .returning(|_, _| Ok(()));
        mock.expect_write_logs()
            .with(eq(time), eq(b"veryveryveryver".to_vec()))
            .times(1)
            .returning(|_, _| Ok(()));
        mock.expect_write_logs()
            .with(eq(time), eq(b"yverylonglonglo".to_vec()))
            .times(1)
            .returning(|_, _| Ok(()));
        mock.expect_write_logs()
            .with(eq(time), eq(b"ngline\n".to_vec()))
            .times(1)
            .returning(|_, _| Ok(()));
        mock.expect_write_logs().times(0);

        let mut writer = LinesWriter::new(mock, 15);
        writer
            .write_logs(time, b"line1\nline2\nline3")
            .await
            .unwrap();
        writer
            .write_logs(
                time,
                b"extra\nveryveryveryveryverylonglonglongline\nbuffered",
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn split_lines2() {
        let mut mock = MockAsyncLogWriter::new();
        let time = SystemTime::now();

        mock.expect_write_logs()
            .with(eq(time), eq(b"foof".to_vec()))
            .times(1)
            .returning(|_, _| Ok(()));
        mock.expect_write_logs()
            .with(eq(time), eq(b"oo\n".to_vec()))
            .times(1)
            .returning(|_, _| Ok(()));
        mock.expect_write_logs()
            .with(eq(time), eq(b"barb".to_vec()))
            .times(1)
            .returning(|_, _| Ok(()));
        mock.expect_write_logs()
            .with(eq(time), eq(b"ar\n".to_vec()))
            .times(1)
            .returning(|_, _| Ok(()));
        mock.expect_write_logs().times(0);

        let mut writer = LinesWriter::new(mock, 4);
        writer.write_logs(time, b"foofoo\nbarbar\n").await.unwrap();
    }
}
