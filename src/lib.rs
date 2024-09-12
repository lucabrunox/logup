mod multiwriter;
mod queuewriter;
mod reader;
mod writer;
mod writer_aws;

use crate::multiwriter::MultiWriter;
use crate::queuewriter::QueueWriter;
use crate::reader::AsyncLogReader;
use crate::writer::AsyncLogWriter;
use crate::writer_aws::{AWSArgs, AWSLogsWriter};
use clap::Parser;
use std::time::SystemTime;
use tokio::task::JoinHandle;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct OutlogArgs {
    #[command(flatten)]
    aws: AWSArgs,
}

pub async fn run(args: OutlogArgs) {
    let mut handles: Vec<JoinHandle<()>> = vec![];

    {
        let mut reader = tokio::io::stdin();
        let mut writers: Vec<Box<dyn AsyncLogWriter + Send>> = vec![];

        writers.push(Box::new(tokio::io::stdout()));
        if let Some((writer, handle)) = AWSLogsWriter::new(&args.aws)
            .await
            .map(|w| QueueWriter::new(w, args.aws.aws_max_memory_items))
        {
            writers.push(Box::new(writer));
            handles.push(handle);
        }

        let mut writer = MultiWriter::new(writers);
        read_and_write_loop(&mut reader, &mut writer).await;
    }

    // ensure everything went out of scope at this point, so that tasks can exit
    for h in handles {
        // TODO: use a timeout
        let _ = h.await;
    }
}

pub async fn read_and_write_loop(
    reader: &mut impl AsyncLogReader,
    writer: &mut impl AsyncLogWriter,
) {
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
