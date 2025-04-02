use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct LoggerConfig {
    pub filter: String,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            filter: "info".to_string(),
        }
    }
}
