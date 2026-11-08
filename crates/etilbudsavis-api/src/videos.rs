//! Videos, recipe ingredients, and likes.

use serde_json::{Value, json};

use crate::client::Client;
use crate::error::Result;
use crate::types::{IngredientsResponse, Video, VideoResponse, VideosResponse};

impl Client {
    /// The video feed, optionally filtered by a search term.
    pub async fn videos(&self, limit: u32, search: Option<&str>) -> Result<Vec<Video>> {
        let mut params = serde_json::Map::new();
        params.insert("limit".to_owned(), json!(limit));
        if let Some(search) = search.filter(|term| !term.trim().is_empty()) {
            params.insert("search".to_owned(), json!(search));
        }
        let response: VideosResponse = self.data("videos", Value::Object(params)).await?;
        Ok(response.data)
    }

    /// One video.
    pub async fn video(&self, id: &str) -> Result<Video> {
        let response: VideoResponse = self.data("video", json!({ "id": id })).await?;
        Ok(response.data)
    }

    /// Recipe ingredients for a video, kept as JSON because the shape varies.
    pub async fn video_ingredients(&self, video_id: &str) -> Result<Vec<Value>> {
        let response: IngredientsResponse = self
            .data("videoIngredients", json!({ "videoPublicId": video_id }))
            .await?;
        Ok(response.data)
    }

    /// Videos the current account liked.
    pub async fn liked_videos(&self) -> Result<Vec<Video>> {
        let response: VideosResponse = self.data("likedVideos", Value::Null).await?;
        Ok(response.data)
    }

    /// Videos the current account uploaded.
    pub async fn my_videos(&self) -> Result<Vec<Video>> {
        self.data("myVideos", Value::Null).await
    }

    /// Likes a video.
    pub async fn like_video(&self, public_id: &str) -> Result<()> {
        self.call_api_unit("video-like", json!({ "publicId": public_id }))
            .await
    }

    /// Removes a like.
    pub async fn unlike_video(&self, public_id: &str) -> Result<()> {
        self.call_api_unit("video-unlike", json!({ "publicId": public_id }))
            .await
    }
}
