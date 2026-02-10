import { useAppStore } from "../stores/appStore";
import { openUrl } from "../lib/tauri";

export function HoverPreview() {
  const hoveredItem = useAppStore((s) => s.hoveredItem);
  const isDoneWorking = useAppStore((s) => s.isDoneWorking);
  const setHoveredItem = useAppStore((s) => s.setHoveredItem);
  const clearHoverTimeout = useAppStore((s) => s.clearHoverTimeout);

  if (!hoveredItem || !isDoneWorking) return null;

  const handleMouseEnter = () => {
    clearHoverTimeout();
  };

  const handleMouseLeave = () => {
    setHoveredItem(null);
  };

  const handleOpen = () => {
    openUrl(hoveredItem.url);
  };

  const jokeText = hoveredItem.description || hoveredItem.title;
  const showTitle = hoveredItem.category !== "joke";

  return (
    <div
      className="fixed inset-x-0 bottom-0 z-40 animate-slide-up px-4 pb-4 cursor-pointer"
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
      onClick={handleOpen}
    >
      <div className="bg-cazz-surface border-2 border-dashed border-cazz-accent shadow-2xl max-w-2xl mx-auto overflow-hidden">
        <div className="bg-cazz-accent/10 border-b border-dashed border-cazz-accent/30 px-3 py-1 flex justify-between items-center">
          <span className="text-[10px] font-mono font-bold text-cazz-accent uppercase tracking-widest">
            [PREVIEW_TERMINAL // CAT:{hoveredItem.category.toUpperCase()}]
          </span>
          <span className="text-[9px] font-mono text-cazz-accent/60">
            ID:{hoveredItem.id.slice(0, 8)}
          </span>
        </div>
        
        <div className="p-5">
          {/* Jokes: large centered text */}
          {hoveredItem.category === "joke" && (
            <div className="text-center font-mono">
              <p className="text-lg text-cazz-text italic leading-relaxed">"{jokeText}"</p>
              <div className="mt-4 pt-4 border-t border-dashed border-cazz-border flex justify-center gap-4 text-[9px] text-cazz-muted uppercase tracking-[0.2em]">
                <span>SRC: {hoveredItem.source}</span>
                <span>[ CLICK TO EXECUTE_OPEN ]</span>
              </div>
            </div>
          )}

          {/* Memes: large image */}
          {hoveredItem.category === "meme" && hoveredItem.thumbnail_url && (
            <div className="text-center">
              <div className="inline-block border border-cazz-border p-1 bg-black mb-3">
                <img
                  src={hoveredItem.thumbnail_url}
                  alt={hoveredItem.title}
                  className="max-h-64 w-auto grayscale-0"
                  onError={(e) => {
                    (e.target as HTMLImageElement).style.display = "none";
                  }}
                />
              </div>
              {showTitle && hoveredItem.title && (
                <p className="text-cazz-text font-mono text-sm uppercase tracking-tight">{hoveredItem.title}</p>
              )}
              <div className="mt-4 pt-4 border-t border-dashed border-cazz-border flex justify-center gap-4 text-[9px] text-cazz-muted uppercase tracking-[0.2em]">
                <span>SRC: {hoveredItem.source}</span>
                <span>[ CLICK TO EXECUTE_OPEN ]</span>
              </div>
            </div>
          )}

          {/* News: image + text */}
          {hoveredItem.category === "news" && (
            <div className="flex gap-6">
              {hoveredItem.thumbnail_url && (
                <div className="flex-shrink-0 border border-cazz-border p-1 bg-black h-fit">
                  <img
                    src={hoveredItem.thumbnail_url}
                    alt={hoveredItem.title}
                    className="w-40 h-32 object-cover"
                    onError={(e) => {
                      (e.target as HTMLImageElement).style.display = "none";
                    }}
                  />
                </div>
              )}
              <div className="flex-1 min-w-0 font-mono">
                {showTitle && hoveredItem.title && (
                  <h3 className="font-bold text-cazz-text text-base mb-3 uppercase tracking-tighter leading-none">{hoveredItem.title}</h3>
                )}
                {hoveredItem.description && (
                  <p className="text-cazz-muted text-[13px] leading-snug line-clamp-4">{hoveredItem.description}</p>
                )}
                <div className="mt-4 pt-4 border-t border-dashed border-cazz-border flex gap-4 text-[9px] text-cazz-muted uppercase tracking-[0.2em]">
                  <span>SRC: {hoveredItem.source}</span>
                  <span>[ CLICK TO EXECUTE_OPEN ]</span>
                </div>
              </div>
            </div>
          )}

          {/* Videos: thumbnail + title */}
          {hoveredItem.category === "video" && (
            <div className="flex gap-6">
              {hoveredItem.thumbnail_url && (
                <div className="flex-shrink-0 border border-cazz-border p-1 bg-black h-fit">
                  <img
                    src={hoveredItem.thumbnail_url}
                    alt={hoveredItem.title}
                    className="w-40 h-24 object-cover"
                    onError={(e) => {
                      (e.target as HTMLImageElement).style.display = "none";
                    }}
                  />
                </div>
              )}
              <div className="flex-1 min-w-0 font-mono">
                {showTitle && hoveredItem.title && (
                  <p className="text-cazz-text font-bold text-base uppercase tracking-tighter leading-tight">{hoveredItem.title}</p>
                )}
                <div className="mt-4 pt-4 border-t border-dashed border-cazz-border flex gap-4 text-[9px] text-cazz-muted uppercase tracking-[0.2em]">
                  <span>SRC: {hoveredItem.source}</span>
                  <span>[ CLICK TO EXECUTE_OPEN ]</span>
                </div>
              </div>
            </div>
          )}

          {/* Gossip: title */}
          {hoveredItem.category === "gossip" && (
            <div className="text-center font-mono">
              <p className="text-cazz-text text-lg font-bold uppercase tracking-tighter italic">
                {showTitle ? hoveredItem.title : "SENSATIONAL_INTEL"}
              </p>
              <div className="mt-4 pt-4 border-t border-dashed border-cazz-border flex justify-center gap-4 text-[9px] text-cazz-muted uppercase tracking-[0.2em]">
                <span>SRC: {hoveredItem.source}</span>
                <span>[ CLICK TO EXECUTE_OPEN ]</span>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
