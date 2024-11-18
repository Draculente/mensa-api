use std::{collections::HashMap, sync::Arc};

use anyhow::anyhow;
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};

use crate::model::{APILocation, Source, SourceData};

pub struct Store<'a> {
    caches: HashMap<String, Arc<Box<dyn Cache<'a>>>>,
}

impl<'a: 'static> Store<'a> {
    pub(crate) fn new_with_ttl(
        ttl: Duration,
        sources: Vec<Box<impl Source<Item = dyn SourceData> + 'static>>,
    ) -> Self {
        let mut caches: HashMap<String, Arc<Box<dyn Cache<'_>>>> = HashMap::new();
        sources
            .into_iter()
            .map(|s| TTLCache::new(s, ttl))
            .map(|c| Box::new(c) as Box<dyn Cache<'_>>)
            .map(|c| Arc::new(c))
            .flat_map(|c| {
                let c = c.clone();
                c.get_locations()
                    .into_iter()
                    .map({
                        let c = c.clone();
                        move |l| (l.code.clone(), c.clone())
                    })
                    .collect::<Vec<_>>()
            })
            .for_each(|entry: (String, Arc<Box<dyn Cache<'a>>>)| {
                caches.insert(entry.0, entry.1);
            });

        Self { caches }
    }
}

#[async_trait(?Send)]
pub(crate) trait Cache<'a> {
    fn get_locations(&self) -> &Vec<APILocation>;
    fn get_data(&self) -> anyhow::Result<&dyn SourceData>;
    fn is_stale(&self) -> bool;
    // fn get_source(&self) -> anyhow::Result<&Box<dyn Source>>;
    async fn fetch(&'a mut self) -> anyhow::Result<()>;
    fn get_last_update_as_string(&self) -> String;
}

#[derive(Debug, Clone)]
pub(crate) struct TTLCache<'a, T>
where
    T: Source<Item = Box<dyn SourceData>>,
{
    source: T,
    data: Option<&'a T::Item>,
    last_updated: DateTime<Utc>,
    ttl: Duration,
}

impl<'a, T: Source<Item = Box<dyn SourceData>>> TTLCache<'a, T> {
    fn new(source: T, ttl: Duration) -> Self {
        println!("Cache initialized with ttl of {ttl}");
        Self {
            source,
            data: None,
            last_updated: DateTime::from_timestamp_nanos(0),
            ttl,
        }
    }
}

#[async_trait(?Send)]
impl<'a, T: Source<Item = dyn SourceData>> Cache<'a> for TTLCache<'a, T> {
    fn get_data(&self) -> anyhow::Result<&dyn SourceData> {
        self.data.ok_or(anyhow!(
            "Failed to get data, because option is empty. This should not have happened!"
        ))
    }

    fn get_locations(&self) -> &Vec<APILocation> {
        self.source.get_locations()
    }

    fn is_stale(&self) -> bool {
        let now = chrono::offset::Utc::now();
        self.last_updated + self.ttl < now
    }

    async fn fetch(&'a mut self) -> anyhow::Result<()> {
        self.data = Some(self.source.fetch().await?);
        self.last_updated = chrono::offset::Utc::now();
        Ok(())
    }

    fn get_last_update_as_string(&self) -> String {
        self.last_updated.to_string()
    }
}
