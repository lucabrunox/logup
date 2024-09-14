mod reader;
mod writer;
mod writer_aws;
mod writer_lines;
mod writer_multi;
mod writer_newrelic;
mod writer_queue;

use crate::reader::AsyncLogReader;
use crate::writer::AsyncLogWriter;
use crate::writer_aws::{AWSArgs, AWSLogsWriter};
use crate::writer_lines::LinesWriter;
use crate::writer_multi::MultiWriter;
use crate::writer_newrelic::{NewRelicArgs, NewRelicWriter};
use crate::writer_queue::QueueWriter;
use clap::Parser;
use std::time::SystemTime;
use tokio::task::JoinHandle;

#[derive(Parser)]
#[command(
    version,
    about = "Find examples on https://github.com/lucabrunox/outlog"
)]
pub struct OutlogArgs {
    #[command(flatten)]
    aws: AWSArgs,

    #[command(flatten)]
    newrelic: NewRelicArgs,

    #[arg(
        long,
        default_value_t = 1000000,
        help = "Force flush without newline beyond the given size"
    )]
    max_line_size: usize,

    #[arg(
        long,
        requires = "aws",
        help = "Max logs to keep in memory before dropping the incoming ones",
        default_value = "1000"
    )]
    max_memory_items: usize,

    #[arg(
        long,
        requires = "aws",
        help = "Max retries before dropping a log",
        default_value = "100"
    )]
    max_retries: u32,
}

pub async fn run(args: OutlogArgs) {
    let mut handles: Vec<JoinHandle<()>> = vec![];

    {
        let mut reader = tokio::io::stdin();

        let mut writers: Vec<Box<dyn AsyncLogWriter + Send>> = vec![];
        if let Some((writer, handle)) = AWSLogsWriter::new(&args.aws, args.max_retries)
            .await
            .map(|w| QueueWriter::new(w, args.max_memory_items))
        {
            writers.push(Box::new(writer));
            handles.push(handle);
        }

        if let Some((writer, handle)) =
            NewRelicWriter::new(&args.newrelic).map(|w| QueueWriter::new(w, args.max_memory_items))
        {
            writers.push(Box::new(writer));
            handles.push(handle);
        }

        let mut writer = MultiWriter::new(vec![
            Box::new(tokio::io::stdout()) as Box<dyn AsyncLogWriter + Send>,
            Box::new(LinesWriter::new(
                MultiWriter::new(writers),
                args.max_line_size,
            )),
        ]);
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
