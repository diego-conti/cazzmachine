use super::provider::{ContentProvider, FetchedItem};
use serde::Deserialize;

pub struct RedditVideoProvider;

#[derive(Deserialize)]
struct RedditListing {
    data: RedditListingData,
}

#[derive(Deserialize)]
struct RedditListingData {
    children: Vec<RedditChild>,
}

#[derive(Deserialize)]
struct RedditChild {
    data: RedditPost,
}

#[derive(Deserialize)]
struct RedditPost {
    title: String,
    permalink: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    thumbnail: String,
    #[serde(default)]
    over_18: bool,
    #[serde(default)]
    stickied: bool,
    #[serde(default)]
    is_video: bool,
}

#[async_trait::async_trait]
impl ContentProvider for RedditVideoProvider {
    fn name(&self) -> &str {
        "reddit-videos"
    }

    fn category(&self) -> &str {
        "video"
    }

    async fn fetch(&self, client: &reqwest::Client) -> Vec<FetchedItem> {
        let subreddits = ["videos", "aww", "AnimalsBeingDerps", "Unexpected"];
        let sub = subreddits[rand::random::<usize>() % subreddits.len()];
        let url = format!("https://www.reddit.com/r/{}/hot.json?limit=15", sub);

        let response = match client
            .get(&url)
            .header("User-Agent", "cazzmachine/0.1.0")
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                log::warn!("Video fetch failed for r/{}: {}", sub, e);
                return vec![];
            }
        };

        let listing: RedditListing = match response.json().await {
            Ok(l) => l,
            Err(e) => {
                log::warn!("Video parse failed for r/{}: {}", sub, e);
                return vec![];
            }
        };

        listing
            .data
            .children
            .into_iter()
            .filter(|c| {
                !c.data.over_18
                    && !c.data.stickied
                    && (c.data.is_video || c.data.url.contains("youtu"))
            })
            .take(5)
            .map(|c| {
                let post = c.data;
                let thumb = if post.thumbnail.starts_with("http") {
                    Some(post.thumbnail)
                } else {
                    None
                };
                FetchedItem {
                    source: format!("r/{}", sub),
                    category: "video".into(),
                    title: post.title,
                    url: format!("https://reddit.com{}", post.permalink),
                    thumbnail_url: thumb,
                    description: None,
                }
            })
            .collect()
    }
}
