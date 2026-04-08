use std::path::Path;

use chrono::NaiveDate;
use serde::Deserialize;

use crate::error::{AppError, Result};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub symbol: String,
    pub interval: String,
    pub pattern_min_length: usize,
    pub pattern_max_length: usize,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

impl Config {
    /// Parses start_date to a Unix timestamp in milliseconds, or None.
    pub fn start_time_ms(&self) -> Result<Option<i64>> {
        self.parse_date_ms(&self.start_date, "start_date")
    }

    /// Parses end_date to a Unix timestamp in milliseconds (end of day), or None.
    pub fn end_time_ms(&self) -> Result<Option<i64>> {
        self.parse_date_ms(&self.end_date, "end_date")
    }

    fn parse_date_ms(&self, field: &Option<String>, name: &str) -> Result<Option<i64>> {
        let Some(date_str) = field else {
            return Ok(None);
        };
        let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .map_err(|e| AppError::Config(format!("Invalid {name} '{date_str}': {e}")))?;
        let timestamp = date
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| AppError::Config(format!("Invalid {name} timestamp")))?
            .and_utc()
            .timestamp_millis();
        Ok(Some(timestamp))
    }
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| AppError::Config(format!("Cannot read config file: {e}")))?;
        let config: Self = toml::from_str(&content)
            .map_err(|e| AppError::Config(format!("Invalid TOML: {e}")))?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        if self.pattern_min_length < 2 {
            return Err(AppError::Config(
                "pattern_min_length must be >= 2".to_owned(),
            ));
        }
        if self.pattern_max_length < self.pattern_min_length {
            return Err(AppError::Config(
                "pattern_max_length must be >= pattern_min_length".to_owned(),
            ));
        }
        if self.pattern_max_length > 10 {
            return Err(AppError::Config(
                "pattern_max_length must be <= 10".to_owned(),
            ));
        }
        Ok(())
    }
}
