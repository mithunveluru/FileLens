import * as Tooltip from "@radix-ui/react-tooltip";
import { AnimatePresence, MotionConfig, motion } from "framer-motion";
import { FolderTree, ScanSearch, Settings as SettingsIcon, Sparkles } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { Toaster, toast } from "sonner";
import { useAnalysis } from "@/features/analysis/useAnalysis";
import Dashboard from "@/features/dashboard/Dashboard";
import OrganizationView from "@/features/organization/OrganizationView";
import ScanPanel from "@/features/scan/ScanPanel";
import { useScan } from "@/features/scan/useScan";
import SettingsModal from "@/features/settings/SettingsModal";
import { useSettings } from "@/features/settings/useSettings";
import { useTheme } from "@/features/settings/useTheme";
import { getAppInfo } from "@/shared/ipc/commands";
import { logger } from "@/shared/logging/logger";
import type { AppInfo, Settings } from "@/shared/types";
import "./App.css";

type View = "cleanup" | "organize";

const VIEWS: Array<{ id: View; label: string; Icon: typeof Sparkles }> = [
  { id: "cleanup", label: "Clean up", Icon: Sparkles },
  { id: "organize", label: "Organize", Icon: FolderTree },
];

function App() {
  const settings = useSettings();
  const scan = useScan();
  const analysis = useAnalysis();
  const [appInfo, setAppInfo] = useState<AppInfo | null>(null);
  const [showSettings, setShowSettings] = useState(false);
  const [view, setView] = useState<View>("cleanup");
  const autoScanned = useRef(false);
  const theme = settings.value?.theme ?? "system";

  useTheme(theme);

  useEffect(() => {
    logger.info("Application started");
    getAppInfo()
      .then(setAppInfo)
      .catch(() => {
        // Error is already logged by the IPC client; the UI degrades gracefully.
      });
  }, []);

  useEffect(() => {
    if (!autoScanned.current && settings.value?.autoScanOnStartup) {
      autoScanned.current = true;
      void scan.start();
    }
  }, [settings.value?.autoScanOnStartup, scan.start]);

  // A finished scan changes the inventory, so re-run the analysis.
  useEffect(() => {
    if (scan.status === "done") void analysis.run();
  }, [scan.status, analysis.run]);

  const handleSaveSettings = async (next: Settings) => {
    await settings.save(next);
    setShowSettings(false);
    toast.success("Settings saved");
    void analysis.run();
  };

  return (
    // `reducedMotion="user"` makes every animation below honour the OS setting.
    <MotionConfig reducedMotion="user">
      <Tooltip.Provider delayDuration={200}>
        <div className="app">
          <header className="app-header">
            <div className="app-brand">
              <span className="app-mark" aria-hidden="true">
                <ScanSearch />
              </span>
              <div>
                <h1>File Lens</h1>
                <p className="tagline">Understand. Organize. Reclaim.</p>
              </div>
            </div>

            <nav className="app-views" aria-label="Workflow">
              {VIEWS.map(({ id, label, Icon }) => (
                <button
                  key={id}
                  type="button"
                  className={view === id ? "active" : ""}
                  aria-pressed={view === id}
                  onClick={() => setView(id)}
                >
                  {/* The pill slides between tabs instead of snapping. */}
                  {view === id && (
                    <motion.span
                      layoutId="view-pill"
                      className="app-view-pill"
                      transition={{ type: "spring", stiffness: 420, damping: 34 }}
                    />
                  )}
                  <span className="app-view-label">
                    <Icon />
                    {label}
                  </span>
                </button>
              ))}
            </nav>

            <button type="button" onClick={() => setShowSettings(true)}>
              <SettingsIcon />
              Settings
            </button>
          </header>

          <main className="app-body">
            <ScanPanel scan={scan} />
            <AnimatePresence mode="wait">
              <motion.div
                key={view}
                className="app-view"
                initial={{ opacity: 0, y: 8 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -8 }}
                transition={{ duration: 0.18, ease: [0.32, 0.72, 0, 1] }}
              >
                {view === "cleanup" ? <Dashboard analysis={analysis} /> : <OrganizationView />}
              </motion.div>
            </AnimatePresence>
          </main>

          {appInfo && (
            <p className="app-footer">
              {appInfo.name} v{appInfo.version}
            </p>
          )}

          {showSettings && settings.value && (
            <SettingsModal
              settings={settings.value}
              onSave={handleSaveSettings}
              onClose={() => setShowSettings(false)}
            />
          )}

          <Toaster theme={theme} position="bottom-right" closeButton richColors />
        </div>
      </Tooltip.Provider>
    </MotionConfig>
  );
}

export default App;
