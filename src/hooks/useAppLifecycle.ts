import { useEffect, useCallback } from 'react';
import { useAppStore } from '../stores/appStore';
import { getLastActiveTimestamp, setLastActiveTimestamp, consumePendingItems, type ConsumeResult, logDiagnostic, isAndroid } from '../lib/tauri';
import { onResume, onPause } from 'tauri-plugin-app-events-api';

export function useAppLifecycle() {
  const {
    threadCount,
    clearStatusTimer,
    setToastMessage
  } = useAppStore();

  const handleAppResume = useCallback(async () => {
    try {
      const lastActive = await getLastActiveTimestamp();
      const now = Date.now();
      const elapsedMinutes = (now - lastActive) / 60000;

      const state = useAppStore.getState();
      void logDiagnostic("doomscroll_trigger", "info", "App resume consumption", {
        source: "app_resume",
        currentStatus: state.systemStatus,
        isFirstRun: state.isFirstRun,
        elapsedSinceLastActive: elapsedMinutes,
      });

      await setLastActiveTimestamp(now);

      if (elapsedMinutes < 1) {
        return;
      }

      const budgetMinutes = elapsedMinutes * threadCount;

      const result = await consumePendingItems(budgetMinutes);

      if (result.items_consumed > 0) {
        const msg = formatResumeMessage(result);
        setToastMessage(msg);
        setTimeout(() => setToastMessage(null), 8000);
      }

      void logDiagnostic("app_resume", "info", `App resume: consumed ${result.items_consumed} items`, { elapsedMinutes: elapsedMinutes.toFixed(1) });
    } catch (error) {
      void logDiagnostic("app_resume_error", "error", "Failed to handle app resume", { error: String(error) });
    }
  }, [threadCount, setToastMessage]);

  const handleAppBackground = useCallback(async () => {
    try {
      const now = Date.now();
      await setLastActiveTimestamp(now);

      clearStatusTimer();
    } catch (error) {
      void logDiagnostic("app_background_error", "error", "Failed to handle app background", { error: String(error) });
    }
  }, [clearStatusTimer]);

  useEffect(() => {
    if (!isAndroid()) {
      const handleVisibilityChange = async () => {
        if (document.visibilityState === 'visible') {
          await handleAppResume();
        } else {
          await handleAppBackground();
        }
      };
      document.addEventListener('visibilitychange', handleVisibilityChange);
      return () => {
        document.removeEventListener('visibilitychange', handleVisibilityChange);
      };
    }

    const setup = async () => {
      try {
        await getLastActiveTimestamp();
      } catch {
        await setLastActiveTimestamp(Date.now());
      }
    };

    setup();

    onResume(handleAppResume);
    onPause(handleAppBackground);

    return () => {
      onResume(() => {});
      onPause(() => {});
    };
  }, [handleAppResume, handleAppBackground]);
}

function formatResumeMessage(result: ConsumeResult): string {
  const parts: string[] = [];
  if (result.memes_consumed > 0) parts.push(`${result.memes_consumed} meme${result.memes_consumed > 1 ? 's' : ''}`);
  if (result.jokes_consumed > 0) parts.push(`${result.jokes_consumed} dad joke${result.jokes_consumed > 1 ? 's' : ''}`);
  if (result.news_consumed > 0) parts.push(`${result.news_consumed} news article${result.news_consumed > 1 ? 's' : ''}`);
  if (result.videos_consumed > 0) parts.push(`${result.videos_consumed} video${result.videos_consumed > 1 ? 's' : ''}`);
  if (result.gossip_consumed > 0) parts.push(`${result.gossip_consumed} gossip piece${result.gossip_consumed > 1 ? 's' : ''}`);

  const summary = parts.length > 0 ? parts.join(', ') : `${result.items_consumed} items`;

  return `While you were away, I doomscrolled through ${summary}. You're welcome.`;
}
