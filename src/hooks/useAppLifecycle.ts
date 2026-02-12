import { useEffect, useCallback } from 'react';
import { useAppStore } from '../stores/appStore';
import { getLastActiveTimestamp, setLastActiveTimestamp, onAndroidAppBackground, consumePendingItems, type ConsumeResult } from '../lib/tauri';

function isAndroid(): boolean {
  return navigator.userAgent.includes('Android');
}

export function useAppLifecycle() {
  const {
    threadCount,
    toggleSystemStatus,
    setToastMessage
  } = useAppStore();

  const handleAppResume = useCallback(async () => {
    try {
      // Get last active timestamp
      const lastActive = await getLastActiveTimestamp();
      const now = Date.now();
      const elapsedMinutes = (now - lastActive) / 60000;

      // Update last active timestamp
      await setLastActiveTimestamp(now);

      // Skip if less than 1 minute has passed
      if (elapsedMinutes < 1) {
        return;
      }

      // Calculate consumption based on elapsed time
      // Formula: budget = elapsed_minutes × items_per_minute × thread_count
      // We need to convert elapsed time to consumption budget
      const budgetMinutes = elapsedMinutes * threadCount;

      // Consume all accumulated items
      const result = await consumePendingItems(budgetMinutes);

      // Show notification for accumulated consumption
      if (result.items_consumed > 0) {
        const msg = formatResumeMessage(result, elapsedMinutes);
        setToastMessage(`[cazz_resume] ${msg}`);
        setTimeout(() => setToastMessage(null), 8000);
      }

      // Start fresh consumption cycle
      toggleSystemStatus();

      console.log(`App resume: consumed ${result.items_consumed} items after ${elapsedMinutes.toFixed(1)} minutes`);
    } catch (error) {
      console.error('Failed to handle app resume:', error);
    }
  }, [threadCount, toggleSystemStatus, setToastMessage]);

  const handleAppBackground = useCallback(async () => {
    try {
      // Update last active timestamp
      const now = Date.now();
      await setLastActiveTimestamp(now);

      // Notify backend (Android only)
      if (isAndroid()) {
        await onAndroidAppBackground();
      }
    } catch (error) {
      console.error('Failed to handle app background:', error);
    }
  }, []);

  useEffect(() => {
    // Set up lifecycle listeners
    const handleVisibilityChange = async () => {
      if (document.visibilityState === 'visible') {
        await handleAppResume();
      } else {
        await handleAppBackground();
      }
    };

    document.addEventListener('visibilitychange', handleVisibilityChange);

    // Android app lifecycle events (if available)
    if (isAndroid()) {
      // These would be set up by the Tauri backend
    }

    // Initial setup
    const setup = async () => {
      // Ensure last active timestamp is set
      try {
        await getLastActiveTimestamp();
      } catch {
        // First run, set initial timestamp
        await setLastActiveTimestamp(Date.now());
      }
    };

    setup();

    return () => {
      document.removeEventListener('visibilitychange', handleVisibilityChange);
    };
  }, [handleAppResume, handleAppBackground]);
}

function formatResumeMessage(result: ConsumeResult, elapsedMinutes: number): string {
  const parts: string[] = [];
  if (result.memes_consumed > 0) parts.push(`${result.memes_consumed} meme${result.memes_consumed > 1 ? 's' : ''}`);
  if (result.jokes_consumed > 0) parts.push(`${result.jokes_consumed} dad joke${result.jokes_consumed > 1 ? 's' : ''}`);
  if (result.news_consumed > 0) parts.push(`${result.news_consumed} news article${result.news_consumed > 1 ? 's' : ''}`);
  if (result.videos_consumed > 0) parts.push(`${result.videos_consumed} video${result.videos_consumed > 1 ? 's' : ''}`);
  if (result.gossip_consumed > 0) parts.push(`${result.gossip_consumed} gossip piece${result.gossip_consumed > 1 ? 's' : ''}`);

  const summary = parts.length > 0 ? parts.join(', ') : `${result.items_consumed} items`;

  const hours = Math.floor(elapsedMinutes / 60);
  const minutes = Math.round(elapsedMinutes % 60);
  const timeStr = hours > 0 ? `${hours}h ${minutes}m` : `${minutes}m`;

  return `While you were away for ${timeStr}, consumed ${summary}. You're welcome.`;
}
