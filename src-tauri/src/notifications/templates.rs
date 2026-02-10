use crate::db::models::{CrawlItem, DayStats};
use rand::seq::SliceRandom;

pub fn generate_teaser(stats: &DayStats, latest: Option<&CrawlItem>) -> String {
    let mut rng = rand::thread_rng();

    if stats.total_items == 0 {
        let idle_messages = [
            "Still warming up the doomscroll engines... You keep working.",
            "Haven't found anything distracting yet. The internet is quiet. Suspicious.",
            "Zero memes so far. Is the internet broken? Either way, keep focusing.",
        ];
        return idle_messages.choose(&mut rng).unwrap().to_string();
    }

    let category_teasers = match latest.map(|i| i.category.as_str()) {
        Some("meme") => vec![
            format!("Just saw the funniest meme. But you've got work to do."),
            format!("Found a meme that made me snort. You can see it later."),
            format!("The memes are fire today. You'll love them. Later though."),
            format!("I've been looking at memes so you don't have to. You're welcome."),
        ],
        Some("joke") => vec![
            format!(
                "Read {} dad jokes. None were funny. You're not missing anything.",
                stats.jokes_found
            ),
            format!(
                "Why did the programmer quit? Because he didn't get arrays. Anyway, keep coding."
            ),
            format!("I just read a joke so bad it looped back around to being good. Focus."),
            format!("Found a dad joke. It's terrible. You'll love it later."),
        ],
        Some("news") => vec![
            format!(
                "Checked the news {} times. Nothing happened. Keep working.",
                stats.news_checked
            ),
            format!("Breaking: absolutely nothing important happened. Stay focused."),
            format!("The news is still depressing. I'm reading it so you don't have to."),
            format!("World still turning. No alien invasion yet. Back to work."),
        ],
        Some("video") => vec![
            format!("Just watched a cat video. It was adorable. You can't see it yet."),
            format!(
                "Found a video that's {} seconds of pure joy. Save it for later.",
                rand::random::<u32>() % 30 + 10
            ),
            format!("The internet has blessed us with another funny video. Keep grinding."),
        ],
        Some("gossip") => vec![
            format!("Celebrity did a thing. It's juicy. But you have deadlines."),
            format!("Entertainment news update: drama happened. You can read about it at 5pm."),
            format!("Someone famous did something dumb. Nothing new. Keep working."),
        ],
        _ => vec![format!(
            "Found {} interesting things while you've been working. Stay focused!",
            stats.total_items
        )],
    };

    let general_teasers = vec![
        format!(
            "I've found {} things today. {} memes, {} jokes. You can binge later.",
            stats.total_items, stats.memes_found, stats.jokes_found
        ),
        format!(
            "Doomscrolled for you: {} items and counting. Estimated {} minutes saved.",
            stats.total_items, stats.estimated_time_saved_minutes as u32
        ),
        format!(
            "Your procrastination proxy is hard at work. {} items catalogued.",
            stats.total_items
        ),
        format!(
            "I've checked {} news articles, {} memes, and {} videos. You've checked zero. Perfect.",
            stats.news_checked, stats.memes_found, stats.videos_found
        ),
    ];

    let use_category = rand::random::<f32>() > 0.4;
    if use_category && !category_teasers.is_empty() {
        category_teasers.choose(&mut rng).unwrap().clone()
    } else {
        general_teasers.choose(&mut rng).unwrap().clone()
    }
}
