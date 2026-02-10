use super::provider::{ContentProvider, FetchedItem};
use serde::Deserialize;

pub struct GoogleNewsRssProvider;

#[derive(Deserialize)]
struct RssResponse {
    items: Vec<RssItem>,
}

#[derive(Deserialize)]
struct RssItem {
    title: String,
    link: String,
    #[serde(default)]
    description: String,
}

#[async_trait::async_trait]
impl ContentProvider for GoogleNewsRssProvider {
    fn name(&self) -> &str {
        "google-news"
    }

    fn category(&self) -> &str {
        "news"
    }

    async fn fetch(&self, client: &reqwest::Client) -> Vec<FetchedItem> {
        let rss_url = "https://news.google.com/rss?hl=en-US&gl=US&ceid=US:en";
        let rss_to_json = format!(
            "https://api.rss2json.com/v1/api.json?rss_url={}",
            urlencoded(rss_url)
        );

        let response = match client
            .get(&rss_to_json)
            .header("User-Agent", "cazzmachine/0.1.0")
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                log::warn!("News RSS fetch failed: {}", e);
                return vec![];
            }
        };

        let rss: RssResponse = match response.json().await {
            Ok(r) => r,
            Err(e) => {
                log::warn!("News RSS parse failed: {}", e);
                return vec![];
            }
        };

        rss.items
            .into_iter()
            .take(8)
            .map(|item| {
                let desc = if item.description.is_empty() {
                    None
                } else {
                    Some(strip_html(&item.description).chars().take(200).collect())
                };
                FetchedItem {
                    source: "Google News".into(),
                    category: "news".into(),
                    title: item.title,
                    url: item.link,
                    thumbnail_url: None,
                    description: desc,
                }
            })
            .collect()
    }
}

fn urlencoded(s: &str) -> String {
    s.replace(':', "%3A").replace('/', "%2F").replace('?', "%3F").replace('=', "%3D").replace('&', "%26")
}

fn strip_html(s: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }
    result
}
