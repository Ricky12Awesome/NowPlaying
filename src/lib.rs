use cached::proc_macro::io_cached;
use youtube_dl::{YoutubeDl, YoutubeDlOutput};

use cache::MetadataCache;
pub use error::*;
use id::YoutubeVideoIdMapper;

use crate::id::Url2Id;

pub mod cache;
pub mod error;
pub mod id;

const VIDEO_URL: &str = "https://www.youtube.com/watch?v=dVteKLjhKFM";

#[io_cached(
  map_error = r##"|e| e"##,
  type = "MetadataCache",
  convert = r#"{ YoutubeVideoIdMapper.try_get_id(url)? }"#,
  create = r##" {
    MetadataCache::new("./cache", false).await.unwrap()
  }"##
)]
async fn get_metadata(url: &str) -> Result<YoutubeDlOutput> {
  YoutubeDl::new(url)
    .socket_timeout("15")
    .run_async()
    .await
    .map_err(Error::YoutubeErr)
}

pub async fn main() {
  let output = get_metadata(VIDEO_URL).await.unwrap();
  let video = output.into_single_video().unwrap();

  println!("Title: {}", video.title);
  println!(
    "Artist: {}",
    video
      .artist
      .or(video.album_artist)
      .or(video.channel)
      .unwrap_or_default()
  );
}
