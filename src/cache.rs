use std::path::PathBuf;

use async_trait::async_trait;
use cached::IOCachedAsync;
use dashmap::DashMap;
use youtube_dl::YoutubeDlOutput;

use crate::{Error, id::Id, Result};

pub struct MetadataCache {
  cache: DashMap<Id, Option<YoutubeDlOutput>>,
  dir: PathBuf,
  refresh: bool,
}

impl MetadataCache {
  pub async fn new(dir: impl Into<PathBuf>, init_cache: bool) -> Result<MetadataCache> {
    let dir = dir.into();
    let cache = DashMap::with_capacity(256);

    if !dir.exists() {
      tokio::fs::create_dir_all(&dir).await?;
    }

    let slf = Self {
      cache,
      dir,
      refresh: false,
    };

    if init_cache {
      slf.init_cache().await?;
    }

    Ok(slf)
  }

  pub async fn init_cache(&self) -> Result<()> {
    for item in self.dir.read_dir().unwrap().filter_map(|e| e.ok()) {
      let path = item.file_name();
      let path = path.to_string_lossy();
      let path = path.trim_end_matches(".json");
      let json = tokio::fs::read_to_string(path).await?;
      let json = serde_json::de::from_str(&json)?;

      self.cache_set(path.into(), json).await?;
    }

    Ok(())
  }
}

#[async_trait]
impl IOCachedAsync<Id, YoutubeDlOutput> for MetadataCache {
  type Error = Error;

  async fn cache_get(&self, k: &Id) -> Result<Option<YoutubeDlOutput>> {
    match self.cache.get(k) {
      None => {
        let path = self.dir.join(format!("{k}.json"));

        if path.exists() {
          let json = tokio::fs::read_to_string(path).await?;
          let data = serde_json::de::from_str(&json)?;

          Ok(Some(data))
        } else {
          Ok(None)
        }
      }
      Some(cached) => Ok(cached.to_owned()),
    }
  }

  async fn cache_set(&self, k: Id, v: YoutubeDlOutput) -> Result<Option<YoutubeDlOutput>> {
    let path = self.dir.join(format!("{k}.json"));
    let json = serde_json::ser::to_string(&v)?;

    tokio::fs::write(path, &json).await?;

    self.cache.insert(k, Some(v.clone()));

    Ok(Some(v))
  }

  async fn cache_remove(&self, k: &Id) -> Result<Option<YoutubeDlOutput>> {
    let path = self.dir.join(format!("{k}.json"));

    if path.exists() {
      tokio::fs::remove_file(path).await?;
    }

    let data = self.cache.remove(k).and_then(|o| o.1);

    Ok(data)
  }

  fn cache_set_refresh(&mut self, refresh: bool) -> bool {
    let old = self.refresh;
    self.refresh = refresh;
    old
  }
}
