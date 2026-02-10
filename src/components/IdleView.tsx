import { useAppStore } from "../stores/appStore";
import { useCrawlStats } from "../hooks/useCrawlStats";
import { ThrottleKnob } from "./ThrottleKnob";

const categoryEmojis: Record<string, string> = {
  meme: "🖼️",
  joke: "😂",
  news: "📰",
  video: "🎬",
  gossip: "💅",
};

export function IdleView() {
  const fetchSummary = useAppStore((s) => s.fetchSummary);
  const peekItems = useAppStore((s) => s.peekItems);
  const stats = useCrawlStats();
  const systemStatus = useAppStore((s) => s.systemStatus);
  const toggleSystemStatus = useAppStore((s) => s.toggleSystemStatus);

  const handleViewDetail = () => {
    peekItems();
  };

  const handleToggleStatus = () => {
    toggleSystemStatus();
  };

  return (
    <div className="animate-fade-in flex flex-col items-center justify-center min-h-[80vh] space-y-8">
      <div className="text-center space-y-4">
        <img 
          src="/doomscrolling.png" 
          alt="doomscrolling" 
          className="w-48 h-48 mx-auto drop-shadow-2xl"
        />
        <h1 className="text-3xl font-black tracking-tighter text-cazz-text uppercase italic">
          cazzmachine
        </h1>
        <p className="text-[10px] font-mono uppercase tracking-[0.2em] text-cazz-muted">
          la macchina che cazzeggia per te
        </p>
      </div>

      {stats && stats.total_items > 0 && (
        <div className="bg-cazz-surface border-2 border-dashed border-cazz-border rounded-xl p-4 w-full max-w-sm">
          <div className="space-y-1">
            {stats.memes_found > 0 && (
              <div className="grid grid-cols-[3rem_1.5rem_1fr_4rem] gap-2 text-[11px] font-mono items-center">
                <span className="text-cazz-text text-right">{stats.memes_found.toString().padStart(3, '0')}</span>
                <span className="text-center">{categoryEmojis.meme}</span>
                <span className="text-cazz-muted uppercase">MEME_LOG</span>
                <span className="text-cazz-muted/60 text-right">-{Math.round(stats.memes_found * 0.5)}m</span>
              </div>
            )}
            {stats.jokes_found > 0 && (
              <div className="grid grid-cols-[3rem_1.5rem_1fr_4rem] gap-2 text-[11px] font-mono items-center">
                <span className="text-cazz-text text-right">{stats.jokes_found.toString().padStart(3, '0')}</span>
                <span className="text-center">{categoryEmojis.joke}</span>
                <span className="text-cazz-muted uppercase">JOKE_BUFFER</span>
                <span className="text-cazz-muted/60 text-right">-{Math.round(stats.jokes_found * 0.3)}m</span>
              </div>
            )}
            {stats.news_checked > 0 && (
              <div className="grid grid-cols-[3rem_1.5rem_1fr_4rem] gap-2 text-[11px] font-mono items-center">
                <span className="text-cazz-text text-right">{stats.news_checked.toString().padStart(3, '0')}</span>
                <span className="text-center">{categoryEmojis.news}</span>
                <span className="text-cazz-muted uppercase">NEWS_FEED</span>
                <span className="text-cazz-muted/60 text-right">-{Math.round(stats.news_checked * 2)}m</span>
              </div>
            )}
            {stats.videos_found > 0 && (
              <div className="grid grid-cols-[3rem_1.5rem_1fr_4rem] gap-2 text-[11px] font-mono items-center">
                <span className="text-cazz-text text-right">{stats.videos_found.toString().padStart(3, '0')}</span>
                <span className="text-center">{categoryEmojis.video}</span>
                <span className="text-cazz-muted uppercase">VIDEO_STREAM</span>
                <span className="text-cazz-muted/60 text-right">-{Math.round(stats.videos_found * 3)}m</span>
              </div>
            )}
            {stats.gossip_found > 0 && (
              <div className="grid grid-cols-[3rem_1.5rem_1fr_4rem] gap-2 text-[11px] font-mono items-center">
                <span className="text-cazz-text text-right">{stats.gossip_found.toString().padStart(3, '0')}</span>
                <span className="text-center">{categoryEmojis.gossip}</span>
                <span className="text-cazz-muted uppercase">GOSSIP_DUMP</span>
                <span className="text-cazz-muted/60 text-right">-{Math.round(stats.gossip_found * 1.5)}m</span>
              </div>
            )}
            <div
              onClick={handleToggleStatus}
              className="mt-3 pt-3 border-t border-dashed border-cazz-border flex justify-between items-center text-[10px] font-mono uppercase cursor-pointer hover:bg-cazz-card/50 transition-colors px-2 -mx-2 rounded"
            >
              <span className={systemStatus === "doomscrolling" ? "text-cazz-accent animate-pulse" : systemStatus === "interrupted" ? "text-red-500" : "text-cazz-muted"}>
                System Status: {systemStatus === "doomscrolling" ? "doomscrolling..." : systemStatus === "interrupted" ? "interrupted" : "standby"}
              </span>
              <span className="text-cazz-muted/60">Saved: {Math.round(stats.estimated_time_saved_minutes)}m</span>
            </div>
          </div>
        </div>
      )}

      <div className="flex flex-col items-center gap-1">
        <span className="text-sm font-mono font-bold uppercase tracking-[0.25em] text-cazz-text">
          Doomscroll Level
        </span>
        <ThrottleKnob />
      </div>

      <div className="flex flex-col items-center gap-4 w-full max-w-sm">
        <button
          onClick={fetchSummary}
          className="w-full py-4 bg-cazz-accent text-white rounded-xl text-xs font-mono font-bold uppercase tracking-[0.2em] hover:bg-cazz-accent/90 transition-all active:scale-[0.98] shadow-lg shadow-cazz-accent/20"
        >
          [ TERMINATE_WORK_SESSION ]
        </button>
        <span
          onClick={handleViewDetail}
          className="text-[10px] font-mono uppercase tracking-[0.2em] text-cazz-muted hover:text-cazz-highlight transition-colors cursor-pointer border-b border-transparent hover:border-cazz-highlight"
        >
          Enter Stealth Preview Mode
        </span>
      </div>
    </div>
  );
}
