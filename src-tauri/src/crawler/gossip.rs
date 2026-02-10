use super::provider::{ContentProvider, FetchedItem};

pub struct GossipProvider;

#[async_trait::async_trait]
impl ContentProvider for GossipProvider {
    fn name(&self) -> &str {
        "gossip"
    }

    fn category(&self) -> &str {
        "gossip"
    }

    async fn fetch(&self, client: &reqwest::Client) -> Vec<FetchedItem> {
        let feeds = [
            "https://www.reddit.com/r/entertainment/hot.json",
            "https://www.reddit.com/r/popculturechat/hot.json",
        ];
        let feed_url = feeds[rand::random::<usize>() % feeds.len()];

        let response = match client
            .get(feed_url)
            .header("User-Agent", "cazzmachine/0.1.0")
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                log::warn!("Gossip fetch failed: {}", e);
                return vec![];
            }
        };

        let listing: serde_json::Value = match response.json().await {
            Ok(v) => v,
            Err(e) => {
                log::warn!("Gossip parse failed: {}", e);
                return vec![];
            }
        };

        let children = match listing["data"]["children"].as_array() {
            Some(c) => c,
            None => return vec![],
        };

        children
            .iter()
            .filter_map(|child| {
                let data = &child["data"];
                let over_18 = data["over_18"].as_bool().unwrap_or(false);
                let stickied = data["stickied"].as_bool().unwrap_or(false);
                if over_18 || stickied {
                    return None;
                }
                let title = data["title"].as_str()?.to_string();
                let permalink = data["permalink"].as_str()?.to_string();
                let thumbnail = data["thumbnail"].as_str().unwrap_or("").to_string();
                let thumb = if thumbnail.starts_with("http") {
                    Some(thumbnail)
                } else {
                    None
                };
                Some(FetchedItem {
                    source: "gossip".into(),
                    category: "gossip".into(),
                    title,
                    url: format!("https://reddit.com{}", permalink),
                    thumbnail_url: thumb,
                    description: None,
                })
            })
            .take(6)
            .collect()
    }
}
