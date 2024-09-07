mod writer;
mod writer_aws;
mod reader;
mod multiwriter;

use crate::reader::AsyncLogReader;
use crate::writer::{AsyncLogWriter};
use crate::writer_aws::{AWSArgs, AWSLogsWriter};
use clap::{Parser};
use std::time::SystemTime;
use crate::multiwriter::MultiWriter;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct OutlogArgs {
    #[command(flatten)]
    aws: AWSArgs,
}

pub async fn run(args: OutlogArgs) {
    let mut reader = tokio::io::stdin();

    let stdout_writer = Some(tokio::io::stdout());
    let aws_writer = AWSLogsWriter::new(args.aws).await;

    let writers = vec![
        stdout_writer.map(|w| Box::new(w) as Box<dyn AsyncLogWriter + Send>),
        aws_writer.map(|w| Box::new(w) as Box<dyn AsyncLogWriter + Send>)
    ].into_iter().flatten().collect();
    let mut writer = MultiWriter::new(writers);

    read_and_write_loop(&mut reader, &mut writer).await
}

pub async fn read_and_write_loop(reader: &mut impl AsyncLogReader, writer: &mut impl AsyncLogWriter) {
    let mut buf: [u8; 1024] = [0; 1024];
    let mut time: SystemTime = SystemTime::now();

    loop {
        let size = reader.read_logs(&mut buf, &mut time).await.unwrap();
        if size == 0 {
            break;
        }
        writer.write_logs(time, &buf[..size]).await.unwrap();
    }
}

fn report_err<E>(err: E)
where
    E: std::error::Error,
    E: Send + Sync,
{
    eprintln!("[ERROR] {}", err);
    if let Some(cause) = err.source() {
        eprintln!("Caused by:");
        for (i, e) in std::iter::successors(Some(cause), |e| e.source()).enumerate() {
            eprintln!("   {}: {}", i, e);
        }
    }
}
