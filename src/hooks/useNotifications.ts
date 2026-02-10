import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { isPermissionGranted, requestPermission } from "@tauri-apps/plugin-notification";
import { useAppStore } from "../stores/appStore";

export function useNotifications() {
  const startDoomscrollingCycle = useAppStore((s) => s.startDoomscrollingCycle);

  useEffect(() => {
    async function setupNotifications() {
      const hasPermission = await isPermissionGranted();
      if (!hasPermission) {
        await requestPermission();
      }
    }
    setupNotifications().catch(console.error);

    // Backend event is just a trigger to start doomscrolling.
    // The only user-facing toast comes at phase end (from appStore).
    const unlisten = listen<string>("cazz-notification", () => {
      startDoomscrollingCycle();
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [startDoomscrollingCycle]);
}
