use anyhow::anyhow;
use chrono::{DateTime, Duration, Utc};

use crate::model::Data;

#[derive(Debug, Clone)]
pub struct Cache {
    data: Option<Data>,
    last_updated: DateTime<Utc>,
    ttl: Duration,
}

impl Cache {
    pub async fn get_data(&self) -> anyhow::Result<&Data> {
        self.data.as_ref().ok_or(anyhow!(
            "Failed to get data, because option is empty. This should not have happened!"
        ))
    }

    pub fn needs_update(&self) -> bool {
        let now = chrono::offset::Utc::now();
        self.last_updated + self.ttl < now
    }

    pub async fn fetch(&mut self) -> anyhow::Result<()> {
        self.data = Some(Data::fetch().await?);
        self.last_updated = chrono::offset::Utc::now();
        Ok(())
    }

    pub fn new(ttl: Duration) -> anyhow::Result<Self> {
        println!("Cache initialized with ttl of {ttl}");
        Ok(Self {
            data: None,
            last_updated: DateTime::from_timestamp_nanos(0),
            ttl,
        })
    }

    pub fn get_last_update_as_string(&self) -> String {
        self.last_updated.to_string()
    }
}
