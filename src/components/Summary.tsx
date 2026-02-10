import { useEffect } from "react";
import { useAppStore } from "../stores/appStore";
import { CrawlItemCard } from "./CrawlItemCard";

export function Summary() {
  const summary = useAppStore((s) => s.summary);
  const setView = useAppStore((s) => s.setView);
  const fetchItems = useAppStore((s) => s.fetchItems);
  const setIsDoneWorking = useAppStore((s) => s.setIsDoneWorking);

  useEffect(() => {
    setIsDoneWorking(true);
  }, [setIsDoneWorking]);

  if (!summary) return null;

  const { stats, summary_text, highlights } = summary;

  const handleViewAll = () => {
    fetchItems();
    setView("detail");
  };

  return (
    <div className="animate-fade-in space-y-6">
      <div className="flex items-center justify-between border-b-2 border-cazz-border pb-4">
        <div>
          <h2 className="text-2xl font-bold tracking-tighter text-cazz-text uppercase italic">
            SESSION_REPORT.LOG
          </h2>
          <p className="text-[9px] font-mono text-cazz-muted uppercase tracking-widest">
            Session: {new Date().toISOString().split('T')[0]} // Status: PROCESSED
          </p>
        </div>
        <button
          onClick={() => {
            setIsDoneWorking(false);
            setView("idle");
          }}
          className="px-3 py-1.5 border border-cazz-border text-[8px] md:text-[10px] font-mono text-cazz-muted hover:text-cazz-accent hover:border-cazz-accent transition-all uppercase whitespace-nowrap overflow-hidden text-ellipsis max-w-[120px]"
        >
          [ EXIT_SESSION ]
        </button>
      </div>

      <div className="bg-cazz-surface border-2 border-dashed border-cazz-border rounded-xl p-5 relative overflow-hidden">
        <div className="absolute top-0 left-0 w-full h-1 bg-cazz-accent/30" />
        <p className="text-sm font-mono text-cazz-text leading-relaxed">
          {summary_text}
        </p>
        <div className="mt-4 pt-3 border-t border-dashed border-cazz-border flex justify-between items-center text-[10px] font-mono text-cazz-muted uppercase tracking-[0.15em]">
          <span>Cazzmachine v0.1</span>
          <span>{new Date().toLocaleDateString()}</span>
        </div>
      </div>

      <div className="grid grid-cols-3 gap-3">
        <StatBox label="MEME_LOG" value={stats.memes_found.toString().padStart(3, '0')} icon="🖼️" />
        <StatBox label="JOKE_BUF" value={stats.jokes_found.toString().padStart(3, '0')} icon="😂" />
        <StatBox label="NEWS_FEED" value={stats.news_checked.toString().padStart(3, '0')} icon="📰" />
        <StatBox label="VID_STRM" value={stats.videos_found.toString().padStart(3, '0')} icon="🎬" />
        <StatBox label="GOSSIP_DMP" value={stats.gossip_found.toString().padStart(3, '0')} icon="💅" />
        <StatBox
          label="TIME_SAVED"
          value={`${Math.round(stats.estimated_time_saved_minutes).toString().padStart(3, '0')}m`}
          icon="⏱️"
        />
      </div>

      {highlights.length > 0 && (
        <div className="space-y-3">
          <h3 className="text-[10px] font-mono font-bold text-cazz-accent uppercase tracking-widest border-b border-dashed border-cazz-accent/30 pb-1">
            [DETECTED_HIGHLIGHTS]
          </h3>
          {highlights.map((item) => (
            <CrawlItemCard key={item.id} item={item} />
          ))}
        </div>
      )}

      <button
        onClick={handleViewAll}
        className="w-full py-4 bg-cazz-surface border-2 border-cazz-accent/40 font-mono text-[10px] md:text-xs font-bold text-cazz-accent hover:bg-cazz-accent hover:text-white uppercase tracking-[0.2em] transition-all"
      >
        [ EXECUTE_VIEW_FULL_DATADUMP ]
      </button>
    </div>
  );
}

function StatBox({
  label,
  value,
  icon,
}: {
  label: string;
  value: number | string;
  icon?: string;
}) {
  return (
    <div className="bg-cazz-surface border border-cazz-border p-3 text-center font-mono transition-all hover:border-cazz-accent/50 group">
      <div className="text-lg mb-1 group-hover:scale-110 transition-transform">{icon}</div>
      <div className="text-lg font-bold text-cazz-text">{value}</div>
      <div className="text-[9px] text-cazz-muted uppercase tracking-widest mt-1">
        [{label}]
      </div>
    </div>
  );
}
