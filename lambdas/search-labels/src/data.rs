use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct SearchLabels {
    pub labels: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VideoInfo {
    pub labels: Vec<String>,
    pub video_url: String,
    pub preview_url: String,
    pub thumbnail_url: String,
}

impl VideoInfo {
    const S3_URL: &'static str = "https://s3.amazonaws.com";

    pub fn new(labels: Vec<String>, video_id: &str) -> Self {
        let video_url = format!("{}/minitube.videos/{}.mp4", Self::S3_URL, video_id);
        let preview_url = format!("{}/minitube.previews/{}.gif", Self::S3_URL, video_id);
        let thumbnail_url = format!("{}/minitube.thumbnails/{}.png", Self::S3_URL, video_id);

        Self {
            labels,
            video_url,
            preview_url,
            thumbnail_url,
        }
    }
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct SearchResult {
    pub videos: Vec<VideoInfo>,
}
