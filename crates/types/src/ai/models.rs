use eyre::bail;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};
use url::Url;

#[derive(Debug, Clone)]
pub enum AiModel {
    DeepseekR1_1_5b,
    DeepseekR1_7b,
    DeepseekR1_8b,
    DeepseekR1_14b,
    DeepseekR1_32b,
    DeepseekR1_70b,
    Custom(String),
}

impl FromStr for AiModel {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().to_lowercase();
        match s.as_str() {
            "deepseek-r1:1.5b" | "1.5b" | "1.5" => Ok(AiModel::DeepseekR1_1_5b),
            "deepseek-r1:7b" | "7b" | "7" => Ok(AiModel::DeepseekR1_7b),
            "deepseek-r1:8b" | "8b" | "8" => Ok(AiModel::DeepseekR1_8b),
            "deepseek-r1:14b" | "14b" | "14" => Ok(AiModel::DeepseekR1_14b),
            "deepseek-r1:32b" | "32b" | "32" => Ok(AiModel::DeepseekR1_32b),
            "deepseek-r1:70b" | "70b" | "70" => Ok(AiModel::DeepseekR1_70b),
            _ => Ok(AiModel::Custom(s)),
        }
    }
}

impl Display for AiModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let modele_str = match self {
            Self::DeepseekR1_1_5b => "deepseek-r1:1.5b",
            Self::DeepseekR1_7b => "deepseek-r1:7b",
            Self::DeepseekR1_8b => "deepseek-r1:8b",
            Self::DeepseekR1_14b => "deepseek-r1:14b",
            Self::DeepseekR1_32b => "deepseek-r1:32b",
            Self::DeepseekR1_70b => "deepseek-r1:70b",
            Self::Custom(model) => model,
        };
        write!(f, "{modele_str}")
    }
}

#[derive(Debug, Clone)]
pub enum AiWebModel {
    DeepseekR1_1_5b,
}

impl AiWebModel {
    pub fn size_in_gb(&self) -> f64 {
        match self {
            Self::DeepseekR1_1_5b => 0.753,
        }
    }

    pub fn all() -> [Self; 1] {
        [Self::DeepseekR1_1_5b]
    }

    pub fn url(&self) -> Url {
        self.into()
    }
}

impl Display for AiWebModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let modele_str = match self {
            Self::DeepseekR1_1_5b => "DeepSeek-R1-Distill-Qwen-1.5B-Q2",
        };
        write!(f, "{modele_str}")
    }
}

impl FromStr for AiWebModel {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().to_lowercase();
        let r = match s.as_str() {
            "DeepSeek-R1-Distill-Qwen-1.5B-Q2" | "deepseek" => Self::DeepseekR1_1_5b,
            _ => bail!("unknown model name"),
        };
        Ok(r)
    }
}

impl From<&AiWebModel> for Url {
    fn from(value: &AiWebModel) -> Self {
        match value {
            AiWebModel::DeepseekR1_1_5b => "https://huggingface.co/unsloth/DeepSeek-R1-Distill-Qwen-1.5B-GGUF/resolve/main/DeepSeek-R1-Distill-Qwen-1.5B-Q2_K.gguf?download=true",
        }
        .parse()
        .unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AiDownloadModel {
    name: String,
    size: f64,
    url: String,
}

impl From<AiWebModel> for AiDownloadModel {
    fn from(value: AiWebModel) -> Self {
        Self {
            name: value.to_string(),
            size: value.size_in_gb(),
            url: value.url().to_string(),
        }
    }
}
