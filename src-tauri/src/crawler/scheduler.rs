use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;

use crate::commands::{THROTTLE_LEVEL};
use crate::db::Database;
use super::provider::ContentProvider;
use super::{reddit, dadjoke, meme, video, gossip, news};

pub struct CrawlScheduler {
    db: Arc<Database>,
    providers: Vec<Box<dyn ContentProvider>>,
    client: reqwest::Client,
    shutdown_rx: watch::Receiver<bool>,
}

impl CrawlScheduler {
    pub fn new(db: Arc<Database>, shutdown_rx: watch::Receiver<bool>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .expect("failed to build HTTP client");

        let providers: Vec<Box<dyn ContentProvider>> = vec![
            Box::new(reddit::RedditProvider::memes()),
            Box::new(reddit::RedditProvider::dad_jokes()),
            Box::new(reddit::RedditProvider::celebrity_gossip()),
            Box::new(dadjoke::DadJokeProvider),
            Box::new(meme::RedditMemeProvider),
            Box::new(video::RedditVideoProvider),
            Box::new(gossip::GossipProvider),
            Box::new(news::GoogleNewsRssProvider),
        ];

        Self {
            db,
            providers,
            client,
            shutdown_rx,
        }
    }

    fn get_crawl_interval(&self) -> Duration {
        let level = THROTTLE_LEVEL.load(std::sync::atomic::Ordering::Relaxed);
        // Level 10: 2 min, Level 1: 15 min
        // Linear: 15 - (level-1)*1.44 = roughly 2 at level 10, 15 at level 1
        let minutes = 15 - (level as u64 - 1) * 1;
        Duration::from_secs(minutes * 60)
    }

    fn get_providers_per_cycle(&self) -> usize {
        let level = THROTTLE_LEVEL.load(std::sync::atomic::Ordering::Relaxed);
        // Level 10: 3 providers, Level 1: 1 provider
        let count = 1 + ((level as usize - 1) * 2) / 9;
        count.min(self.providers.len())
    }

    pub async fn run(&self) {
        log::info!("CrawlScheduler started with {} providers", self.providers.len());
        let initial_interval = self.get_crawl_interval();
        log::info!("Initial crawl interval: {} seconds", initial_interval.as_secs());

        loop {
            // Crawl when buffer is low (below threshold) to maintain healthy supply
            match self.db.get_pending_count() {
                Ok(n) if n < 20 => {
                    self.crawl_cycle().await;
                }
                Ok(pending) => {
                    log::info!("Buffer has {} unconsumed items - skipping crawl to save bandwidth", pending);
                }
                Err(e) => {
                    log::warn!("Failed to check pending count: {}", e);
                }
            }

            let interval = self.get_crawl_interval();
            let providers_count = self.get_providers_per_cycle();
            log::info!("Next crawl in {} seconds with {} providers", interval.as_secs(), providers_count);

            let mut rx = self.shutdown_rx.clone();
            tokio::select! {
                _ = tokio::time::sleep(interval) => {},
                _ = rx.changed() => {
                    if *rx.borrow() {
                        log::info!("CrawlScheduler shutting down");
                        break;
                    }
                }
            }
        }
    }

    async fn crawl_cycle(&self) {
        let provider_idx = rand::random::<usize>() % self.providers.len();
        let count = self.get_providers_per_cycle();
        let mut items_added = 0u32;

        for i in 0..count {
            let idx = (provider_idx + i) % self.providers.len();
            let provider = &self.providers[idx];
            log::info!("Crawling: {} ({})", provider.name(), provider.category());

            let fetched = provider.fetch(&self.client).await;
            for item in fetched {
                let crawl_item = item.into_crawl_item();
                match self.db.insert_item(&crawl_item) {
                    Ok(true) => items_added += 1,
                    Ok(false) => {} // duplicate
                    Err(e) => log::warn!("Failed to insert item: {}", e),
                }
            }
        }

        log::info!("Crawl cycle complete: {} new items added", items_added);
    }
}
