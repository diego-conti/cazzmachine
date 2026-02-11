import { create } from "zustand";
import type { CrawlItem, DayStats, DaySummary, Category, ConsumeResult } from "../lib/tauri";
import {
  getTodayItems,
  getItemsByCategory,
  getTodayStats,
  getDailySummary,
  setThrottleLevel as apiSetThrottleLevel,
  setConsumptionThreads as apiSetConsumptionThreads,
  consumePendingItems,
  pruneOldItems,
} from "../lib/tauri";

type View = "idle" | "summary" | "detail";

function getDoomscrollDurationMs(level: number): number {
  // Scroll duration: 1→5 min linearly
  // Formula: S(level) = 1 + 4 × (level-1)/9
  const scrollMinutes = 1 + 4 * ((level - 1) / 8);
  return scrollMinutes * 60 * 1000;
}

function getThreadDurations(budgetMs: number, threadCount: number): number[] {
  if (threadCount === 1) return [budgetMs];
  const half = budgetMs / 2;
  const durations: number[] = [];
  for (let i = 0; i < threadCount; i++) {
    durations.push(half + (i * half) / (threadCount - 1));
  }
  return durations;
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
  threadCount: number;
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
  setThreadCount: (count: number) => void;
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
  _launchConsumptionThreads: () => void;
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
  threadCount: 1,
  hoverTimeout: null,
  systemStatus: "standby",
  statusTimer: null,
  interruptedTimer: null,

  toggleSystemStatus: () => {
    const { systemStatus, clearStatusTimer, clearInterruptedTimer } = get();

    if (systemStatus === "standby") {
      clearInterruptedTimer();
      get()._launchConsumptionThreads();
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
    const { systemStatus, statusTimer, clearInterruptedTimer } = get();

    if (systemStatus === "standby" && !statusTimer) {
      clearInterruptedTimer();
      get()._launchConsumptionThreads();
    }
  },

  _launchConsumptionThreads: () => {
    const { throttleLevel, threadCount } = get();
    set({ systemStatus: "doomscrolling" });

    const durationMs = getDoomscrollDurationMs(throttleLevel);
    const threadDurations = getThreadDurations(durationMs, threadCount);
    let completed = 0;
    const timerIds: NodeJS.Timeout[] = [];

    for (let t = 0; t < threadDurations.length; t++) {
      const threadDurationMs = threadDurations[t];
      const threadNum = t + 1;
      const threadBudget = threadDurationMs / 60000;

      const timerId = setTimeout(async () => {
        const result = await consumePendingItems(threadBudget);
        completed++;

        const { setToastMessage, fetchStats } = get();
        await fetchStats();

        const msg = generatePhaseEndToast(result);
        if (msg) {
          setToastMessage(`[Thread ${threadNum}/${threadCount}] ${msg}`);
          setTimeout(() => setToastMessage(null), 5000);
        }

        if (completed >= threadCount) {
          set({ systemStatus: "standby", statusTimer: null });
        }
      }, threadDurationMs);

      timerIds.push(timerId);
    }

    set({ statusTimer: timerIds as any });
  },

  clearStatusTimer: () => {
    const { statusTimer } = get();
    if (statusTimer) {
      if (Array.isArray(statusTimer)) {
        statusTimer.forEach((t) => clearTimeout(t));
      } else {
        clearTimeout(statusTimer);
      }
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
    const clamped = Math.max(1, Math.min(9, level));
    set({ throttleLevel: clamped });
    await apiSetThrottleLevel(clamped);
  },

  setThreadCount: async (count: number) => {
    const clamped = Math.max(1, Math.min(8, count));
    set({ threadCount: clamped });
    await apiSetConsumptionThreads(clamped);
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
