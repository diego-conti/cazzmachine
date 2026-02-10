use crate::db::models::{DayStats, DaySummary};
use crate::db::Database;
use rand::seq::SliceRandom;
use std::sync::Arc;

pub fn generate_daily_summary(db: &Arc<Database>) -> Result<DaySummary, String> {
    let stats = db.get_today_stats().map_err(|e| e.to_string())?;
    let items = db.get_items_for_today().map_err(|e| e.to_string())?;

    let highlights: Vec<_> = items.into_iter().take(5).collect();
    let summary_text = build_summary_text(&stats);

    Ok(DaySummary {
        stats,
        summary_text,
        highlights,
    })
}

fn build_summary_text(stats: &DayStats) -> String {
    let mut rng = rand::thread_rng();

    if stats.total_items == 0 {
        return "I haven't started doomscrolling yet. Give me a minute.".into();
    }

    let time_saved = stats.estimated_time_saved_minutes;
    let hours = (time_saved / 60.0) as u32;
    let minutes = (time_saved % 60.0) as u32;
    let time_str = if hours > 0 {
        format!("{} hours and {} minutes", hours, minutes)
    } else {
        format!("{} minutes", minutes)
    };

    let mut parts: Vec<String> = Vec::new();

    if stats.memes_found > 0 {
        let meme_phrases = [
            format!("doomscrolled {} memes", stats.memes_found),
            format!(
                "browsed {} memes (some were actually funny)",
                stats.memes_found
            ),
            format!("stared at {} memes with dead eyes", stats.memes_found),
        ];
        parts.push(meme_phrases.choose(&mut rng).unwrap().clone());
    }

    if stats.jokes_found > 0 {
        let joke_phrases = [
            format!("read {} jokes (most were terrible)", stats.jokes_found),
            format!("suffered through {} dad jokes", stats.jokes_found),
            format!("groaned at {} unfunny jokes", stats.jokes_found),
        ];
        parts.push(joke_phrases.choose(&mut rng).unwrap().clone());
    }

    if stats.news_checked > 0 {
        let news_phrases = [
            format!(
                "checked the news {} times (nothing happened)",
                stats.news_checked
            ),
            format!(
                "read {} news articles (the world is still a mess)",
                stats.news_checked
            ),
            format!(
                "monitored {} news stories so you don't have to",
                stats.news_checked
            ),
        ];
        parts.push(news_phrases.choose(&mut rng).unwrap().clone());
    }

    if stats.videos_found > 0 {
        let video_phrases = [
            format!(
                "watched {} videos (at least 2 had cats)",
                stats.videos_found
            ),
            format!("found {} videos worth watching later", stats.videos_found),
            format!(
                "sat through {} videos so you could be productive",
                stats.videos_found
            ),
        ];
        parts.push(video_phrases.choose(&mut rng).unwrap().clone());
    }

    if stats.gossip_found > 0 {
        let gossip_phrases = [
            format!("kept up with {} celebrity stories", stats.gossip_found),
            format!("read {} pieces of entertainment gossip", stats.gossip_found),
            format!("followed {} celebrity dramas", stats.gossip_found),
        ];
        parts.push(gossip_phrases.choose(&mut rng).unwrap().clone());
    }

    let activities = if parts.is_empty() {
        "scrolled the internet aimlessly".into()
    } else {
        parts.join(", ")
    };

    let closers = [
        format!(
            "You saved {} of wasted time by letting me handle it.",
            time_str
        ),
        format!("That's {} of your life I saved. You're welcome.", time_str),
        format!(
            "Without me, you'd have wasted {}. I accept payment in compliments.",
            time_str
        ),
    ];

    let opener = format!("Today while you worked, I {}", activities);
    let closer = closers.choose(&mut rng).unwrap();

    format!("{}. {}", opener, closer)
}
