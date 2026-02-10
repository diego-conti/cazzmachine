use super::provider::{ContentProvider, FetchedItem};
use serde::Deserialize;

pub struct RedditMemeProvider;

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
    post_hint: Option<String>,
    #[serde(default)]
    over_18: bool,
    #[serde(default)]
    stickied: bool,
}

#[async_trait::async_trait]
impl ContentProvider for RedditMemeProvider {
    fn name(&self) -> &str {
        "reddit-memes"
    }

    fn category(&self) -> &str {
        "meme"
    }

    async fn fetch(&self, client: &reqwest::Client) -> Vec<FetchedItem> {
        let subreddits = ["dankmemes", "me_irl", "funny", "wholesomememes"];
        let sub = subreddits[rand::random::<usize>() % subreddits.len()];
        let url = format!("https://www.reddit.com/r/{}/hot.json?limit=10", sub);

        let response = match client
            .get(&url)
            .header("User-Agent", "cazzmachine/0.1.0")
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                log::warn!("Meme fetch failed for r/{}: {}", sub, e);
                return vec![];
            }
        };

        let listing: RedditListing = match response.json().await {
            Ok(l) => l,
            Err(e) => {
                log::warn!("Meme parse failed for r/{}: {}", sub, e);
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
                    && c.data.post_hint.as_deref() == Some("image")
            })
            .take(6)
            .map(|c| {
                let post = c.data;
                let image_url = if post.url.starts_with("http") {
                    Some(post.url.clone())
                } else {
                    None
                };
                FetchedItem {
                    source: format!("r/{}", sub),
                    category: "meme".into(),
                    title: post.title,
                    url: format!("https://reddit.com{}", post.permalink),
                    thumbnail_url: image_url,
                    description: None,
                }
            })
            .collect()
    }
}
