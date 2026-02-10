use super::provider::{ContentProvider, FetchedItem};
use serde::Deserialize;

pub struct DadJokeProvider;

#[derive(Deserialize)]
struct DadJokeSearchResponse {
    results: Vec<DadJoke>,
}

#[derive(Deserialize)]
struct DadJoke {
    id: String,
    joke: String,
}

#[async_trait::async_trait]
impl ContentProvider for DadJokeProvider {
    fn name(&self) -> &str {
        "icanhazdadjoke"
    }

    fn category(&self) -> &str {
        "joke"
    }

    async fn fetch(&self, client: &reqwest::Client) -> Vec<FetchedItem> {
        let topics = ["work", "computer", "office", "coffee", "cat", "dog", "food", "money"];
        let topic = topics[rand::random::<usize>() % topics.len()];

        let response = match client
            .get(format!("https://icanhazdadjoke.com/search?term={}&limit=5", topic))
            .header("Accept", "application/json")
            .header("User-Agent", "cazzmachine/0.1.0")
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                log::warn!("DadJoke fetch failed: {}", e);
                return vec![];
            }
        };

        let search: DadJokeSearchResponse = match response.json().await {
            Ok(s) => s,
            Err(e) => {
                log::warn!("DadJoke parse failed: {}", e);
                return vec![];
            }
        };

        search
            .results
            .into_iter()
            .map(|joke| FetchedItem {
                source: "icanhazdadjoke".into(),
                category: "joke".into(),
                title: joke.joke.clone(),
                url: format!("https://icanhazdadjoke.com/j/{}", joke.id),
                thumbnail_url: None,
                description: Some(joke.joke),
            })
            .collect()
    }
}
