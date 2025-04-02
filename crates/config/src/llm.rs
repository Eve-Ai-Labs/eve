use crate::url::{deserialize_url, serialize_url};
use serde::{Deserialize, Serialize};
use url::Url;
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct OllamaConfig {
    #[serde(serialize_with = "serialize_url", deserialize_with = "deserialize_url")]
    pub url: Url,
    pub model: String,
    /// throttle requests per time (time_millis), default 1
    pub req_per_time: u64,
    /// max tokens per request, default 1
    ///
    /// 1 token - 1 parallel request (not llm token)
    pub max_tokens: u64,
    /// time in milliseconds for rate limiting, default 1000
    ///
    /// more info: https://docs.rs/ratelimit/latest/ratelimit/struct.Ratelimiter.html#method.builder
    pub time_millis: u64,
    /// failed request retry limit, default 13
    pub retry_limit: usize,
    /// timeout for Ollama request in seconds, default 300
    pub timeout: u64,
    pub pull_model: bool,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:11434".parse().expect("Never"),
            model: "deepseek-r1:latest".to_string(),
            req_per_time: 1,
            max_tokens: 1,
            time_millis: 1000,
            retry_limit: 13,
            timeout: 300,
            pull_model: false,
        }
    }
}
