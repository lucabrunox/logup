use crate::writer::AsyncLogWriter;
use async_trait::async_trait;
use clap::{Args, ValueEnum};
use std::time::SystemTime;

#[derive(Args)]
#[group()]
pub struct NewRelicArgs {
    #[arg(
        long,
        help = "Enable uploading logs to NewRelic",
        requires = "newrelic_region",
        requires = "newrelic_api_key"
    )]
    newrelic: bool,

    #[arg(
        value_enum,
        value_name = "NEW_RELIC_REGION",
        long,
        env = "NEW_RELIC_REGION",
        hide_env_values = true,
        requires = "newrelic"
    )]
    newrelic_region: Option<NewRelicRegion>,

    #[arg(
        long,
        value_name = "NEW_RELIC_API_KEY",
        env = "NEW_RELIC_API_KEY",
        hide_env_values = true,
        requires = "newrelic"
    )]
    newrelic_api_key: Option<String>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
#[clap(rename_all = "UPPER")]
enum NewRelicRegion {
    US,
    EU,
}

pub struct NewRelicWriter {
    client: reqwest::Client,
    endpoint: String,
    api_key: String,
}

impl NewRelicWriter {
    pub fn new(args: &NewRelicArgs) -> Option<Self> {
        if !args.newrelic {
            return None;
        }

        Some(Self {
            client: reqwest::Client::new(),
            endpoint: match args.newrelic_region.as_ref()? {
                NewRelicRegion::US => "https://log-api.newrelic.com/log/v1".to_string(),
                NewRelicRegion::EU => "https://log-api.eu.newrelic.com/log/v1".to_string(),
            },
            api_key: args.newrelic_api_key.as_ref()?.to_string(),
        })
    }
}

#[async_trait]
impl AsyncLogWriter for NewRelicWriter {
    async fn write_logs(&mut self, time: SystemTime, buf: &[u8]) -> std::io::Result<()> {
        let json = serde_json::json!({
            "timestamp": time.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis(),
            "message": String::from_utf8_lossy(buf).to_string()
        });
        self.client
            .post(&self.endpoint)
            .header("Api-Key", &self.api_key)
            .json(&json)
            .send()
            .await
            .unwrap();
        Ok(())
    }
}
