import { invoke } from "@tauri-apps/api/core";

export interface CrawlItem {
  id: string;
  source: string;
  category: string;
  title: string;
  url: string;
  thumbnail_url: string | null;
  description: string | null;
  fetched_at: string;
  is_seen: boolean;
  is_saved: boolean;
  session_date: string;
}

export interface DayStats {
  memes_found: number;
  jokes_found: number;
  news_checked: number;
  videos_found: number;
  gossip_found: number;
  total_items: number;
  estimated_time_saved_minutes: number;
}

export interface DaySummary {
  stats: DayStats;
  summary_text: string;
  highlights: CrawlItem[];
}

export interface ConsumeResult {
  items_consumed: number;
  items_discarded: number;
  time_consumed_minutes: number;
  memes_consumed: number;
  jokes_consumed: number;
  news_consumed: number;
  videos_consumed: number;
  gossip_consumed: number;
}

export type Category = "meme" | "joke" | "news" | "video" | "gossip";

export async function getTodayItems(): Promise<CrawlItem[]> {
  return invoke<CrawlItem[]>("get_today_items");
}

export async function getItemsByCategory(
  category: Category,
): Promise<CrawlItem[]> {
  return invoke<CrawlItem[]>("get_items_by_category", { category });
}

export async function getTodayStats(): Promise<DayStats> {
  return invoke<DayStats>("get_today_stats");
}

export async function getDailySummary(): Promise<DaySummary> {
  return invoke<DaySummary>("get_daily_summary");
}

export async function toggleSaveItem(itemId: string): Promise<boolean> {
  return invoke<boolean>("toggle_save_item", { itemId });
}

export async function markItemSeen(itemId: string): Promise<void> {
  return invoke<void>("mark_item_seen", { itemId });
}

export async function openUrl(url: string): Promise<void> {
  return invoke<void>("open_url", { url });
}

export async function getThrottleLevel(): Promise<number> {
  return invoke<number>("get_throttle_level");
}

export async function setThrottleLevel(level: number): Promise<void> {
  return invoke<void>("set_throttle_level", { level });
}

export async function consumePendingItems(budgetMinutes: number): Promise<ConsumeResult> {
  return invoke<ConsumeResult>("consume_pending_items", { budgetMinutes });
}

export async function pruneOldItems(): Promise<{ deleted: number; stripped: number }> {
  return invoke<[number, number]>("prune_old_items").then(([deleted, stripped]) => ({ deleted, stripped }));
}
