pub mod provider;
pub mod reddit;
pub mod dadjoke;
pub mod news;
pub mod meme;
pub mod video;
pub mod gossip;
pub mod util;
pub mod jokeapi;
pub mod uselessfacts;
pub mod chucknorris;
pub mod hackernews;
pub mod bbcnews;

use crate::commands::{THROTTLE_LEVEL, THREAD_COUNT};

pub fn providers_per_cycle() -> usize {
    let level = THROTTLE_LEVEL.load(std::sync::atomic::Ordering::Relaxed);
    let thread_count = THREAD_COUNT.load(std::sync::atomic::Ordering::Relaxed) as usize;
    let count = 2 + ((level as usize - 1) * 4) / 8;
    let scaled = count * thread_count.div_ceil(4);
    scaled.min(13)
}
