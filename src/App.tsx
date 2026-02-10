import { useEffect } from "react";
import { useAppStore } from "./stores/appStore";
import { useNotifications } from "./hooks/useNotifications";
import { Toast } from "./components/Toast";
import { HoverPreview } from "./components/HoverPreview";
import { IdleView } from "./components/IdleView";
import { Summary } from "./components/Summary";
import { DetailView } from "./components/DetailView";
import { getThrottleLevel } from "./lib/tauri";

function App() {
  useNotifications();
  const view = useAppStore((s) => s.view);
  const setThrottleLevel = useAppStore((s) => s.setThrottleLevel);

  useEffect(() => {
    getThrottleLevel().then(setThrottleLevel);
  }, [setThrottleLevel]);

  return (
    <div className="min-h-screen bg-cazz-bg text-cazz-text">
      <Toast />
      <HoverPreview />
      <div className="max-w-2xl mx-auto px-6 py-5">
        {view === "idle" && <IdleView />}
        {view === "summary" && <Summary />}
        {view === "detail" && <DetailView />}
      </div>
    </div>
  );
}

export default App;
