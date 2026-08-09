import { useState } from "react";
import type { FormEvent } from "react";
import { useNavigate } from "react-router-dom";

import { usePanelStore } from "../../stores/panelStore";
import type { Panel } from "../../lib/tauri";

interface PanelFormProps {
  editing?: Panel;
}

export default function PanelForm({ editing }: PanelFormProps) {
  const navigate = useNavigate();
  const { add, remove } = usePanelStore();

  const [name, setName] = useState(editing?.name ?? "");
  const [url, setUrl] = useState(editing?.url ?? "");
  const [panelType, setPanelType] = useState<"bt" | "1panel">(
    editing?.panel_type ?? "bt",
  );
  const [error, setError] = useState<string | null>(null);

  async function handleSubmit(e: FormEvent) {
    e.preventDefault();
    setError(null);
    try {
      await add({
        name,
        url,
        panel_type: panelType,
      } as never);
      navigate("/panels");
    } catch (err) {
      setError(String(err));
    }
  }

  return (
    <div className="max-w-xl p-6">
      <h1 className="mb-6 text-xl font-semibold">
        {editing ? "编辑面板" : "添加面板"}
      </h1>

      <form onSubmit={handleSubmit} className="flex flex-col gap-4">
        <label className="flex flex-col gap-1 text-sm">
          <span className="text-zinc-400">面板名称</span>
          <input
            className="rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 outline-none focus:border-indigo-500"
            value={name}
            onChange={(e) => setName(e.target.value)}
            required
          />
        </label>

        <label className="flex flex-col gap-1 text-sm">
          <span className="text-zinc-400">面板地址</span>
          <input
            className="rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 outline-none focus:border-indigo-500"
            value={url}
            onChange={(e) => setUrl(e.target.value)}
            placeholder="https://panel.example.com:8888"
            required
          />
        </label>

        <label className="flex flex-col gap-1 text-sm">
          <span className="text-zinc-400">面板类型</span>
          <select
            className="rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 outline-none focus:border-indigo-500"
            value={panelType}
            onChange={(e) => setPanelType(e.target.value as "bt" | "1panel")}
          >
            <option value="bt">宝塔 (BT)</option>
            <option value="1panel">1Panel</option>
          </select>
        </label>

        {error && <p className="text-sm text-red-400">{error}</p>}

        <div className="flex gap-2">
          <button
            type="submit"
            className="rounded-md bg-indigo-500 px-4 py-2 text-sm font-medium text-white transition hover:bg-indigo-400"
          >
            {editing ? "保存修改" : "添加面板"}
          </button>
          {editing && (
            <button
              type="button"
              onClick={async () => {
                if (confirm(`删除面板 ${editing.name}？`)) {
                  await remove(editing.id);
                  navigate("/panels");
                }
              }}
              className="rounded-md border border-red-500/40 px-4 py-2 text-sm text-red-400 transition hover:bg-red-500/10"
            >
              删除
            </button>
          )}
        </div>
      </form>
    </div>
  );
}
