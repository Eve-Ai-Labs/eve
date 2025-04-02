use crypto::ed25519::public::PublicKey;
use dashmap::DashMap;
use poem::{http::StatusCode, web::headers::RetryAfter, Response};
use ratelimit::Ratelimiter;
use std::{net::IpAddr, sync::Arc, time::Duration};

#[derive(Clone)]
pub struct LimitsMap {
    req_per_hour: u64,
    ip_limits: Arc<DashMap<IpAddr, Ratelimiter>>,
    pubkey_limits: Arc<DashMap<PublicKey, Ratelimiter>>,
}

impl LimitsMap {
    pub fn new(req_per_hour: u64) -> Self {
        Self {
            req_per_hour,
            ip_limits: Default::default(),
            pubkey_limits: Default::default(),
        }
    }
    pub fn pubkey_check(&self, pubkey: &PublicKey) -> Result<(), poem::Error> {
        match self.pubkey_limits.get_mut(pubkey) {
            Some(mut entry) => {
                let value = entry.value_mut();
                value.try_wait().map_err(|duration| {
                    let duration = Duration::from_secs(duration.as_secs());
                    tracing::error!(%pubkey, ?duration, "max requests per hour reached");
                    let resp = Response::builder()
                        .status(StatusCode::TOO_MANY_REQUESTS)
                        .typed_header(RetryAfter::delay(duration))
                        .finish();
                    poem::Error::from_response(resp)
                })
            }
            None => {
                let limiter = Ratelimiter::builder(self.req_per_hour, Duration::from_secs(3600))
                    .max_tokens(self.req_per_hour)
                    .initial_available(self.req_per_hour - 1)
                    .build()
                    .unwrap();
                self.pubkey_limits.insert(*pubkey, limiter);
                Ok(())
            }
        }
    }
    pub fn ip_check(&self, ip_addr: &IpAddr) -> Result<(), poem::Error> {
        match self.ip_limits.get_mut(ip_addr) {
            Some(mut entry) => {
                let value = entry.value_mut();
                value.try_wait().map_err(|duration| {
                    let duration = Duration::from_secs(duration.as_secs());
                    tracing::error!(%ip_addr, "max requests per hour reached");
                    let resp = Response::builder()
                        .status(StatusCode::TOO_MANY_REQUESTS)
                        .typed_header(RetryAfter::delay(duration))
                        .finish();
                    poem::Error::from_response(resp)
                })
            }
            None => {
                let limiter = Ratelimiter::builder(self.req_per_hour, Duration::from_secs(3600))
                    .max_tokens(self.req_per_hour)
                    .initial_available(self.req_per_hour - 1)
                    .build()
                    .unwrap();
                self.ip_limits.insert(*ip_addr, limiter);
                Ok(())
            }
        }
    }
}
