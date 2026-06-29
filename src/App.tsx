import { useEffect, useRef, useState } from "react";
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

/**
 * Composition root. Owns the scan, analysis, and settings controllers so they
 * can coordinate: auto-scan on startup, re-analyze after a scan or a settings
 * change, and apply the chosen theme.
 */
function App() {
  const settings = useSettings();
  const scan = useScan();
  const analysis = useAnalysis();
  const [appInfo, setAppInfo] = useState<AppInfo | null>(null);
  const [showSettings, setShowSettings] = useState(false);
  const [view, setView] = useState<View>("cleanup");
  const autoScanned = useRef(false);

  useTheme(settings.value?.theme ?? "system");

  useEffect(() => {
    logger.info("Application started");
    getAppInfo()
      .then(setAppInfo)
      .catch(() => {
        // Error is already logged by the IPC client; the UI degrades gracefully.
      });
  }, []);

  // Auto-scan once on startup if the user enabled it.
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
    void analysis.run();
  };

  return (
    <main className="app">
      <header className="app-header">
        <div>
          <h1>Download Doctor</h1>
          <p className="tagline">A smarter way to manage your Downloads folder.</p>
        </div>
        <button type="button" onClick={() => setShowSettings(true)}>
          Settings
        </button>
      </header>

      <nav className="app-views" aria-label="Workflow">
        <button
          type="button"
          className={view === "cleanup" ? "active" : ""}
          aria-pressed={view === "cleanup"}
          onClick={() => setView("cleanup")}
        >
          Clean up
        </button>
        <button
          type="button"
          className={view === "organize" ? "active" : ""}
          aria-pressed={view === "organize"}
          onClick={() => setView("organize")}
        >
          Organize
        </button>
      </nav>

      <ScanPanel scan={scan} />
      {view === "cleanup" ? <Dashboard analysis={analysis} /> : <OrganizationView />}

      {appInfo && (
        <p className="version">
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
    </main>
  );
}

export default App;
