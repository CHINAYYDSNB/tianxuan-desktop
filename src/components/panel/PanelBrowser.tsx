import { usePanelBrowserStore } from "../../stores/panelBrowserStore";

export default function PanelBrowser() {
  const { active, tabs, activeTab, switchTab, closeTab, exit } = usePanelBrowserStore();

  if (!active) return null;

  return (
    <div className="fixed inset-x-0 top-0 z-50 flex h-10 items-center gap-1 border-b border-zinc-800 bg-zinc-950 px-2">
      {/* logo: return to app UI but keep tabs alive */}
      <button
        onClick={() => exit()}
        title="返回 Tianxuan（面板保留）"
        className="mr-1 flex h-7 w-7 items-center justify-center rounded-md text-indigo-400 transition hover:bg-zinc-800"
      >
        🛰
      </button>

      <div className="flex min-w-0 flex-1 items-center gap-1 overflow-x-auto">
        {tabs.map((t) => (
          <div
            key={t.label}
            onClick={() => switchTab(t.label)}
            className={`group flex h-7 shrink-0 cursor-pointer items-center gap-1.5 rounded-md px-2 text-xs transition ${
              activeTab === t.label
                ? "bg-indigo-500/15 text-indigo-300"
                : "text-zinc-500 hover:bg-zinc-800 hover:text-zinc-300"
            }`}
          >
            <span className="truncate">{t.name}</span>
            <button
              onClick={(e) => {
                e.stopPropagation();
                closeTab(t.label);
              }}
              className="flex h-4 w-4 items-center justify-center rounded text-zinc-500 hover:bg-zinc-700 hover:text-zinc-200"
            >
              ×
            </button>
          </div>
        ))}
        {tabs.length === 0 && (
          <span className="text-xs text-zinc-600">无打开的面板</span>
        )}
      </div>

      <button
        onClick={() => exit()}
        className="ml-1 rounded-md border border-zinc-700 px-2 py-1 text-xs text-zinc-400 transition hover:border-indigo-500 hover:text-zinc-200"
      >
        返回
      </button>
    </div>
  );
}
