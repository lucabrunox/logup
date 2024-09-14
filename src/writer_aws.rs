use crate::report_err;
use crate::writer::AsyncLogWriter;
use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_sdk_cloudwatchlogs::config::retry::RetryConfig;
use aws_sdk_cloudwatchlogs::types::InputLogEvent;
use aws_sdk_cloudwatchlogs::Client;
use clap::Args;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Args)]
#[group()]
pub struct AWSArgs {
    #[arg(
        long,
        help = "Enable uploading logs to AWS Logs",
        requires = "aws_log_group_name"
    )]
    aws: bool,

    #[arg(
        long,
        requires = "aws",
        env = "AWS_LOG_GROUP_NAME",
        hide_env_values = true
    )]
    aws_log_group_name: Option<String>,

    #[arg(
        long,
        requires = "aws",
        env = "AWS_LOG_STREAM_NAME",
        hide_env_values = true,
        help = "Log stream name [default: hostname]"
    )]
    aws_log_stream_name: Option<String>,
}

pub struct AWSLogsWriter {
    client: Client,
    log_group_name: String,
    log_stream_name: String,
}

impl AWSLogsWriter {
    pub async fn new(args: &AWSArgs, max_retries: u32) -> Option<Self> {
        if !args.aws {
            return None;
        }

        let log_group_name = args.aws_log_group_name.clone().unwrap();
        let log_stream_name = args
            .aws_log_stream_name
            .clone()
            .unwrap_or_else(|| hostname::get().unwrap().into_string().unwrap());
        let client = aws_sdk_cloudwatchlogs::Client::new(
            &aws_config::defaults(BehaviorVersion::latest())
                .retry_config(
                    RetryConfig::standard().with_max_attempts(max_retries + 1), // initial call is included
                )
                .load()
                .await,
        );
        create_log_group(&client, &log_group_name).await;
        create_log_stream(&client, &log_group_name, &log_stream_name).await;
        let writer = Self {
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
