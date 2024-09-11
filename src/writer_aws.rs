use crate::report_err;
use crate::writer::AsyncLogWriter;
use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_sdk_cloudwatchlogs::types::InputLogEvent;
use aws_sdk_cloudwatchlogs::Client;
use clap::Args;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Args)]
#[group()]
pub struct AWSArgs {
    #[arg(long)]
    aws_log_group_name: Option<String>,
    #[arg(long, requires = "aws_log_group_name")]
    aws_log_stream_name: Option<String>,
}

pub struct AWSLogsWriter {
    client: Client,
    log_group_name: String,
    log_stream_name: String,
}

impl AWSLogsWriter {
    pub async fn new(aws_args: AWSArgs) -> Option<AWSLogsWriter> {
        aws_args.aws_log_group_name.as_ref()?;

        let log_group_name = aws_args.aws_log_group_name.unwrap();
        let log_stream_name = aws_args
            .aws_log_stream_name
            .unwrap_or_else(|| hostname::get().unwrap().into_string().unwrap());
        let client = aws_sdk_cloudwatchlogs::Client::new(
            &aws_config::defaults(BehaviorVersion::latest()).load().await,
        );
        create_log_group(&client, &log_group_name).await;
        create_log_stream(&client, &log_group_name, &log_stream_name).await;
        let writer = AWSLogsWriter {
            client,
            log_group_name,
            log_stream_name,
        };
        Some(writer)
    }
}

#[async_trait]
impl AsyncLogWriter for AWSLogsWriter {
    async fn write_logs(&mut self, time: SystemTime, buf: &[u8]) -> std::io::Result<()> {
        let timestamp = time.duration_since(UNIX_EPOCH).unwrap().as_millis() as i64;

        self.client
            .put_log_events()
            .log_group_name(&self.log_group_name)
            .log_stream_name(&self.log_stream_name)
            .set_log_events(Some(vec![InputLogEvent::builder()
                .timestamp(timestamp)
                .message(String::from_utf8_lossy(buf).to_string())
                .build()
                .unwrap()]))
            .send()
            .await
            .unwrap();

        Ok(())
    }
}

macro_rules! is_resource_already_exists_exception {
    ($err:expr) => {
        $err.as_service_error()
            .map(|se| se.is_resource_already_exists_exception())
            .unwrap_or(false)
    };
}

async fn create_log_stream(client: &Client, log_group_name: &str, log_stream_name: &str) {
    let resp = client
        .create_log_stream()
        .log_group_name(log_group_name)
        .log_stream_name(log_stream_name)
        .send()
        .await;

    if let Err(e) = resp {
        if !is_resource_already_exists_exception!(e) {
            report_err(&e);
            panic!(
                "Error creating log stream {}:log-stream:{}: {}",
                log_group_name, log_stream_name, e
            );
        }
    }
}

async fn create_log_group(client: &Client, log_group_name: &str) {
    let resp = client
        .create_log_group()
        .log_group_name(log_group_name)
        .send()
        .await;

    if let Err(e) = resp {
        if !is_resource_already_exists_exception!(e) {
            report_err(&e);
            panic!("Error creating log group: {}", e);
        }
    }
}
