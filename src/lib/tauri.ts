import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-shell";

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

declare global {
  interface Window {
    Tauri?: any;
  }
}

export async function openUrl(url: string): Promise<void> {
  try {
    await open(url);
  } catch (e) {
    console.error("Failed to open URL with shell plugin:", e);
    
    if (typeof window !== 'undefined' && window.Tauri) {
      try {
        await window.Tauri.invoke('open_url', { url });
      } catch (backendError) {
        console.error("Failed to open URL with backend command:", backendError);
        
        if (url.startsWith('http')) {
          window.open(url, '_blank');
        }
      }
    } else {
      if (url.startsWith('http')) {
        window.open(url, '_blank');
      }
    }
  }
}

export async function getThrottleLevel(): Promise<number> {
  return invoke<number>("get_throttle_level");
}

export async function setThrottleLevel(level: number): Promise<void> {
  return invoke<void>("set_throttle_level", { level });
}

export async function getConsumptionThreads(): Promise<number> {
  return invoke<number>("get_consumption_threads");
}

export async function setConsumptionThreads(count: number): Promise<void> {
  return invoke<void>("set_consumption_threads", { count });
}

export async function getPendingCount(): Promise<number> {
  return invoke<number>("get_pending_count");
}

export async function consumePendingItems(budgetMinutes: number): Promise<ConsumeResult> {
  return invoke<ConsumeResult>("consume_pending_items", { budgetMinutes });
}

export async function pruneOldItems(): Promise<{ deleted: number; stripped: number }> {
  return invoke<[number, number]>("prune_old_items").then(([deleted, stripped]) => ({ deleted, stripped }));
}
