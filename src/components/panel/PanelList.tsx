import { useEffect } from "react";
import { Link } from "react-router-dom";

import { usePanelStore } from "../../stores/panelStore";
import { usePanelBrowserStore } from "../../stores/panelBrowserStore";

export default function PanelList() {
  const { panels, loading, error, load } = usePanelStore();
  const enter = usePanelBrowserStore((s) => s.enter);

  useEffect(() => {
    load();
  }, [load]);

  async function openPanel(id: string, name: string) {
    try {
      await enter(id, name);
    } catch (e) {
      alert(String(e));
    }
  }

  return (
    <div className="p-6">
      <div className="mb-6 flex items-center justify-between">
        <h1 className="text-xl font-semibold">面板管理</h1>
        <Link
          to="/panels/new"
          className="rounded-md bg-indigo-500 px-4 py-2 text-sm font-medium text-white transition hover:bg-indigo-400"
        >
          + 添加面板
        </Link>
      </div>

      {error && <p className="mb-4 text-sm text-red-400">{error}</p>}
      {loading && <p className="text-sm text-zinc-500">加载中...</p>}

      {!loading && panels.length === 0 && (
        <div className="rounded-lg border border-dashed border-zinc-700 p-10 text-center text-sm text-zinc-500">
          还没有面板，点击右上角「添加面板」开始
        </div>
      )}

      <div className="flex flex-col gap-2">
        {panels.map((p) => (
          <div
            key={p.id}
            className="flex items-center gap-4 rounded-lg border border-zinc-800 bg-zinc-950 p-3"
          >
            <div className="flex h-9 w-9 items-center justify-center rounded-md bg-zinc-800 text-sm">
              🌐
            </div>
            <div className="min-w-0 flex-1">
              <div className="flex items-center gap-2">
                <span className="font-medium">{p.name}</span>
                <span className="rounded bg-indigo-500/15 px-1.5 py-0.5 text-[10px] text-indigo-300">
                  {p.panel_type === "bt" ? "宝塔" : "1Panel"}
                </span>
              </div>
              <div className="text-xs text-zinc-500">{p.url}</div>
            </div>
            <div className="flex gap-2">
              <button
                onClick={() => openPanel(p.id, p.name)}
                className="rounded-md border border-zinc-700 px-3 py-1.5 text-xs text-indigo-300 transition hover:border-indigo-500"
              >
                打开
              </button>
              <Link
                to={`/panels/${p.id}/edit`}
                className="rounded-md border border-zinc-700 px-3 py-1.5 text-xs text-zinc-300 transition hover:border-indigo-500"
              >
                编辑
              </Link>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
