use super::provider::{ContentProvider, FetchedItem};
use serde::Deserialize;

fn is_image_url(url: &str) -> bool {
    url.ends_with(".jpg")
        || url.ends_with(".jpeg")
        || url.ends_with(".png")
        || url.ends_with(".gif")
        || url.ends_with(".webp")
        || url.contains("i.redd.it")
        || url.contains("i.imgur.com")
        || url.contains("preview.redd.it")
}

pub struct RedditProvider {
    subreddit: String,
    category: String,
}

impl RedditProvider {
    pub fn memes() -> Self {
        Self {
            subreddit: "memes".into(),
            category: "meme".into(),
        }
    }

    pub fn dad_jokes() -> Self {
        Self {
            subreddit: "dadjokes".into(),
            category: "joke".into(),
        }
    }

    pub fn celebrity_gossip() -> Self {
        Self {
            subreddit: "entertainment".into(),
            category: "gossip".into(),
        }
    }
}

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
    selftext: String,
    #[serde(default)]
    over_18: bool,
    #[serde(default)]
    stickied: bool,
    #[serde(default)]
    is_self: bool,
}

#[async_trait::async_trait]
impl ContentProvider for RedditProvider {
    fn name(&self) -> &str {
        &self.subreddit
    }

    fn category(&self) -> &str {
        &self.category
    }

    async fn fetch(&self, client: &reqwest::Client) -> Vec<FetchedItem> {
        let url = format!(
            "https://www.reddit.com/r/{}/hot.json?limit=10",
            self.subreddit
        );

        let response = match client
            .get(&url)
            .header("User-Agent", "cazzmachine/0.1.0")
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                log::warn!("Reddit fetch failed for r/{}: {}", self.subreddit, e);
                return vec![];
            }
        };

        let listing: RedditListing = match response.json().await {
            Ok(l) => l,
            Err(e) => {
                log::warn!("Reddit parse failed for r/{}: {}", self.subreddit, e);
                return vec![];
            }
        };

        listing
            .data
            .children
            .into_iter()
            .filter(|c| !c.data.over_18 && !c.data.stickied)
            .take(8)
            .map(|c| {
                let post = c.data;
                let thumbnail = if !post.is_self && is_image_url(&post.url) {
                    Some(post.url.clone())
                } else if post.thumbnail.starts_with("http") && post.thumbnail != "default" && post.thumbnail != "self" && post.thumbnail != "nsfw" {
                    Some(post.thumbnail)
                } else {
                    None
                };
                let description = if post.selftext.is_empty() {
                    None
                } else {
                    Some(post.selftext.chars().take(200).collect())
                };
                FetchedItem {
                    source: format!("r/{}", self.subreddit),
                    category: self.category.clone(),
                    title: post.title,
                    url: format!("https://reddit.com{}", post.permalink),
                    thumbnail_url: thumbnail,
                    description,
                }
            })
            .collect()
    }
}
