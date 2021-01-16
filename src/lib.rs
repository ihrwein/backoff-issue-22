extern crate backoff;

use backoff::ExponentialBackoff;
use std::time::Duration;

use anyhow::Result;
use reqwest::Response;

#[macro_use]
extern crate async_trait;

#[async_trait]
pub trait SendRetry {
    async fn send_retry(
        self,
        retry_period: Duration,
        max_elapsed_time: Duration,
    ) -> Result<Response>;
}

#[async_trait]
impl SendRetry for reqwest::RequestBuilder {
    async fn send_retry(
        self,
        retry_period: Duration,
        max_elapsed_time: Duration,
    ) -> Result<Response> {
        let op = || async {
            self.try_clone()
                .ok_or(backoff::Error::Permanent(anyhow::Error::msg(
                    "this request cannot be cloned",
                )))?
                .send()
                .await
                .map_err(|err| backoff::Error::Transient(anyhow::Error::from(err)))
        };

        backoff::tokio::retry(
            ExponentialBackoff {
                current_interval: retry_period,
                initial_interval: retry_period,
                max_elapsed_time: Some(max_elapsed_time),
                ..ExponentialBackoff::default()
            },
            op,
        )
        .await
    }
}
