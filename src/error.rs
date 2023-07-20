use std::convert::Infallible;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
#[error("{0}")]
pub enum Error {
  #[error("No id found")]
  NoIdFount,
  UrlParseErr(#[from] url::ParseError),
  AnyhowErr(#[from] anyhow::Error),
  YoutubeErr(#[from] youtube_dl::Error),
  JsonErr(#[from] serde_json::Error),
  Infallible(#[from] Infallible),
  Io(#[from] std::io::Error),
  Custom(String),
}


