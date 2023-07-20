use std::{borrow::Cow, convert::Infallible, ops::Not, str::FromStr, sync::Arc};

use derive_more::{AsRef, Deref, Display, From};
use serde::{Deserialize, Serialize};
use url::{ParseError, Url};

use crate::{Error, Result};

#[derive(
  Serialize, Deserialize, Display, Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Deref, AsRef, From,
)]
pub struct Id(Arc<str>);

macro_rules! impl_id {
  ($($t:ty $(:$a:lifetime)?),+) => {
    $(impl $(<$a>)? From<$t> for Id {
      fn from(value: $t) -> Self {
        Self(Arc::from(value))
      }
    })*
  };
}

unsafe impl Send for Id {}
unsafe impl Sync for Id {}

impl_id!(String, &'_ str, Cow<'_, str>);

impl FromStr for Id {
  type Err = Infallible;

  fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
    Ok(Self(Arc::from(s)))
  }
}

impl Id {
  pub fn as_str(&self) -> &str {
    self.0.as_ref()
  }
}

pub trait Url2IdMapper: 'static {
  fn get_id(&self, url: &Url) -> Result<Id>;

  fn to_dyn(&self) -> &dyn Url2IdMapper
  where
    Self: Sized,
  {
    self
  }
}

pub trait Url2Id {
  fn try_get_id(&self, url: impl TryInto<Url, Error = ParseError>) -> Result<Id>;
}

impl<T: Url2IdMapper> Url2Id for T {
  fn try_get_id(&self, url: impl TryInto<Url, Error = ParseError>) -> Result<Id> {
    let url = url.try_into()?;

    self.get_id(&url)
  }
}

impl Url2Id for &[&dyn Url2IdMapper] {
  fn try_get_id(&self, url: impl TryInto<Url, Error = ParseError>) -> Result<Id> {
    let url = url.try_into()?;

    self
      .iter()
      .find_map(|t| t.get_id(&url).ok())
      .ok_or(Error::NoIdFount)
  }
}

impl<const N: usize> Url2Id for [&dyn Url2IdMapper; N] {
  fn try_get_id(&self, url: impl TryInto<Url, Error = ParseError>) -> Result<Id> {
    self.as_ref().try_get_id(url)
  }
}

pub struct YoutubeVideoIdMapper;

impl Url2IdMapper for YoutubeVideoIdMapper {
  fn get_id(&self, url: &Url) -> Result<Id> {
    match (url.host_str(), url.path()) {
      (Some("www.youtube.com" | "youtube.com"), "/watch") => url
        .query_pairs()
        .find_map(|(key, value)| key.eq("v").then_some(value))
        .map(Into::into)
        .ok_or(Error::NoIdFount),
      (Some("youtu.be"), id) => id
        .is_empty()
        .not()
        .then(|| id.trim_start_matches('/').into())
        .ok_or(Error::NoIdFount),
      _ => Err(Error::NoIdFount),
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::id::{Url2Id, YoutubeVideoIdMapper};

  #[test]
  fn test() {
    let short_url = "https://youtu.be/dVteKLjhKFM";
    let long_url = "https://www.youtube.com/watch?v=dVteKLjhKFM";

    let value = YoutubeVideoIdMapper.try_get_id(short_url).unwrap();
    println!("{value:?}");

    let value = YoutubeVideoIdMapper.try_get_id(long_url).unwrap();
    println!("{value:?}");
  }
}
