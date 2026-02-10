import { create } from "zustand";
import type { CrawlItem, DayStats, DaySummary, Category, ConsumeResult } from "../lib/tauri";
import {
  getTodayItems,
  getItemsByCategory,
  getTodayStats,
  getDailySummary,
  setThrottleLevel as apiSetThrottleLevel,
  consumePendingItems,
  pruneOldItems,
} from "../lib/tauri";

type View = "idle" | "summary" | "detail";

function getDoomscrollDurationMs(level: number): number {
  return (60 + ((level - 1) / 9) * 240) * 1000;
}

function generatePhaseEndToast(r: ConsumeResult): string {
  if (r.items_consumed === 0) {
    return "No content found. The internet is quiet... suspiciously quiet.";
  }

  const parts: string[] = [];
  if (r.memes_consumed > 0) parts.push(`${r.memes_consumed} meme${r.memes_consumed > 1 ? "s" : ""}`);
  if (r.jokes_consumed > 0) parts.push(`${r.jokes_consumed} dad joke${r.jokes_consumed > 1 ? "s" : ""}`);
  if (r.news_consumed > 0) parts.push(`${r.news_consumed} news article${r.news_consumed > 1 ? "s" : ""}`);
  if (r.videos_consumed > 0) parts.push(`${r.videos_consumed} video${r.videos_consumed > 1 ? "s" : ""}`);
  if (r.gossip_consumed > 0) parts.push(`${r.gossip_consumed} gossip piece${r.gossip_consumed > 1 ? "s" : ""}`);

  const summary = parts.length > 0 ? parts.join(", ") : `${r.items_consumed} items`;

  const templates = [
    `Doomscrolled ${summary} so you don't have to. You're welcome.`,
    `Just inhaled ${summary}. Your productivity is safe. For now.`,
    `Consumed ${summary}. The internet never sleeps and neither do I.`,
    `${summary} catalogued. Your procrastination proxy delivers.`,
    `Finished binging ${summary}. Back to pretending to work.`,
    `${r.items_consumed} distractions neutralized. ${summary} absorbed.`,
    `Another haul: ${summary}. You owe me a coffee.`,
  ];

  return templates[Math.floor(Math.random() * templates.length)];
}

interface AppState {
  view: View;
  items: CrawlItem[];
  stats: DayStats | null;
  summary: DaySummary | null;
  activeCategory: Category | "all";
  toastMessage: string | null;
  isLoading: boolean;
  isDoneWorking: boolean;
  hoveredItem: CrawlItem | null;
  throttleLevel: number;
  systemStatus: "standby" | "doomscrolling" | "interrupted";
  statusTimer: NodeJS.Timeout | null;
  interruptedTimer: NodeJS.Timeout | null;

  setView: (view: View) => void;
  peekItems: () => void;
  setActiveCategory: (cat: Category | "all") => void;
  setToastMessage: (msg: string | null) => void;
  setIsDoneWorking: (done: boolean) => void;
  setHoveredItem: (item: CrawlItem | null) => void;
  setThrottleLevel: (level: number) => void;
  hoverTimeout: NodeJS.Timeout | null;
  setHoverTimeout: (timeout: NodeJS.Timeout | null) => void;
  clearHoverTimeout: () => void;
  checkNewDay: () => void;
  fetchItems: () => Promise<void>;
  fetchItemsByCategory: (cat: Category) => Promise<void>;
  fetchStats: () => Promise<void>;
  fetchSummary: () => Promise<void>;
  toggleSystemStatus: () => void;
  clearStatusTimer: () => void;
  clearInterruptedTimer: () => void;
  startDoomscrollingCycle: () => void;
}

const getTodayKey = () => new Date().toISOString().split('T')[0];

let currentDay = getTodayKey();

export const useAppStore = create<AppState>((set, get) => ({
  view: "idle",
  items: [],
  stats: null,
  summary: null,
  activeCategory: "all",
  toastMessage: null,
  isLoading: false,
  isDoneWorking: false,
  hoveredItem: null,
  throttleLevel: 5,
  hoverTimeout: null,
  systemStatus: "standby",
  statusTimer: null,
  interruptedTimer: null,

  toggleSystemStatus: () => {
    const { systemStatus, clearStatusTimer, clearInterruptedTimer, throttleLevel } = get();

    if (systemStatus === "standby") {
      clearInterruptedTimer();
      set({ systemStatus: "doomscrolling" });

      const duration = getDoomscrollDurationMs(throttleLevel);
      const budgetMinutes = duration / 60000;
      const timer = setTimeout(async () => {
        const { setToastMessage, fetchStats, clearStatusTimer } = get();
        clearStatusTimer();

        const result = await consumePendingItems(budgetMinutes);
        await fetchStats();

        const msg = generatePhaseEndToast(result);
        if (msg) {
          setToastMessage(msg);
          setTimeout(() => setToastMessage(null), 8000);
        }

        set({ systemStatus: "standby", statusTimer: null });
      }, duration);

      set({ statusTimer: timer });
    } else if (systemStatus === "doomscrolling") {
      clearStatusTimer();
      set({ systemStatus: "interrupted", statusTimer: null });

      const intTimer = setTimeout(async () => {
        const { fetchStats } = get();
        await fetchStats();
        set({ systemStatus: "standby", interruptedTimer: null });
      }, 2000);

      set({ interruptedTimer: intTimer });
    }
  },

  startDoomscrollingCycle: () => {
    const { systemStatus, statusTimer, clearInterruptedTimer, throttleLevel } = get();

    if (systemStatus === "standby" && !statusTimer) {
      clearInterruptedTimer();
      set({ systemStatus: "doomscrolling" });

      const duration = getDoomscrollDurationMs(throttleLevel);
      const budgetMinutes = duration / 60000;
      const timer = setTimeout(async () => {
        const { setToastMessage, fetchStats, clearStatusTimer } = get();
        clearStatusTimer();

        const result = await consumePendingItems(budgetMinutes);
        await fetchStats();

        const msg = generatePhaseEndToast(result);
        if (msg) {
          setToastMessage(msg);
          setTimeout(() => setToastMessage(null), 8000);
        }

        set({ systemStatus: "standby", statusTimer: null });
      }, duration);

      set({ statusTimer: timer });
    }
  },

  clearStatusTimer: () => {
    const { statusTimer } = get();
    if (statusTimer) {
      clearTimeout(statusTimer);
      set({ statusTimer: null });
    }
  },

  clearInterruptedTimer: () => {
    const { interruptedTimer } = get();
    if (interruptedTimer) {
      clearTimeout(interruptedTimer);
      set({ interruptedTimer: null });
    }
  },

  checkNewDay: async () => {
    const today = getTodayKey();
    if (today !== currentDay) {
      currentDay = today;
      // Clean up old data: delete unconsumed, strip consumed to URLs only
      await pruneOldItems().catch(() => {});
      set({
        items: [],
        stats: null,
        summary: null,
      });
    }
  },

  setThrottleLevel: async (level: number) => {
    const clamped = Math.max(1, Math.min(10, level));
    set({ throttleLevel: clamped });
    await apiSetThrottleLevel(clamped);
  },

  setView: (view) => set({ view }),
  peekItems: () => {
    get().fetchItems();
    set({ view: "detail", isDoneWorking: false });
  },
  setActiveCategory: (cat) => {
    set({ activeCategory: cat });
    if (cat === "all") {
      get().fetchItems();
    } else {
      get().fetchItemsByCategory(cat);
    }
  },
  setToastMessage: (msg) => set({ toastMessage: msg }),
  setIsDoneWorking: (done) => set({ isDoneWorking: done }),
  setHoveredItem: (item) => set({ hoveredItem: item }),
  setHoverTimeout: (timeout) => set({ hoverTimeout: timeout }),
  clearHoverTimeout: () => {
    const { hoverTimeout } = get();
    if (hoverTimeout) {
      clearTimeout(hoverTimeout);
      set({ hoverTimeout: null });
    }
  },

  fetchItems: async () => {
    get().checkNewDay();
    set({ isLoading: true });
    try {
      const items = await getTodayItems();
      set({ items, isLoading: false });
    } catch {
      set({ isLoading: false });
    }
  },

  fetchItemsByCategory: async (cat) => {
    get().checkNewDay();
    set({ isLoading: true });
    try {
      const items = await getItemsByCategory(cat);
      set({ items, isLoading: false });
    } catch {
      set({ isLoading: false });
    }
  },

  fetchStats: async () => {
    try {
      const stats = await getTodayStats();
      set({ stats });
    } catch {
      // noop
    }
  },

  fetchSummary: async () => {
    get().checkNewDay();
    set({ isLoading: true });
    try {
      const summary = await getDailySummary();
      set({ summary, view: "summary", isLoading: false });
    } catch {
      set({ isLoading: false });
    }
  },
}));
