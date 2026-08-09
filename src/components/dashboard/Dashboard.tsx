import { useEffect, useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";

import { useHostStore } from "../../stores/hostStore";
import { collectMetrics, type HostMetrics } from "../../lib/tauri";
import { useVisibilityPolling } from "../../hooks/useVisibilityPolling";
import HostForm from "../host/HostForm";

function ProgressBar({ value, color }: { value: number; color: string }) {
  const clamped = Math.max(0, Math.min(100, value));
  return (
    <div className="h-1.5 w-full overflow-hidden rounded-full bg-zinc-800">
      <div
        className={`h-full rounded-full ${color} transition-all duration-700`}
        style={{ width: `${clamped}%` }}
      />
    </div>
  );
}

function MetricRow({
  label,
  text,
  percent,
  color,
}: {
  label: string;
  text: string;
  percent: number;
  color: string;
}) {
  return (
    <div className="flex flex-col gap-1">
      <div className="flex items-center justify-between text-xs">
        <span className="text-zinc-500">{label}</span>
        <span className="font-mono text-zinc-300">{text}</span>
      </div>
      <ProgressBar value={percent} color={color} />
    </div>
  );
}

function HostCard({ id, onEdit }: { id: string; onEdit: (id: string) => void }) {
  const navigate = useNavigate();
  const host = useHostStore((s) => s.hosts.find((h) => h.id === id));
  const [metrics, setMetrics] = useState<HostMetrics | null>(null);
  const [initializing, setInitializing] = useState(true);
  const [fetchedOnce, setFetchedOnce] = useState(false);

  useVisibilityPolling(async () => {
    try {
      const m = await collectMetrics(id);
      setMetrics(m);
      setFetchedOnce(true);
    } catch {
      setFetchedOnce(true);
    } finally {
      setInitializing(false);
    }
  });

  if (!host) return null;

  const online = metrics?.online ?? false;
  const color = online ? "bg-emerald-400" : "bg-zinc-600";

  return (
    <div
      onClick={() => navigate(`/hosts/${host.id}/workspace`)}
      className="group cursor-pointer rounded-lg border border-zinc-800 bg-zinc-950 p-4 transition hover:border-indigo-500/60 hover:bg-zinc-900"
    >
      <div className="mb-3 flex items-center justify-between">
        <div className="min-w-0">
          <div className="flex items-center gap-2">
            <span className="truncate font-medium">{host.name}</span>
            {host.tags.slice(0, 2).map((t) => (
              <span
                key={t}
                className="rounded bg-indigo-500/15 px-1.5 py-0.5 text-[10px] text-indigo-300"
              >
                {t}
              </span>
            ))}
          </div>
          <div className="text-xs text-zinc-500">
            {host.username}@{host.address}
          </div>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={(e) => {
              e.stopPropagation();
              onEdit(host.id);
            }}
            title="编辑主机"
            className="flex h-6 w-6 items-center justify-center rounded text-xs text-zinc-600 opacity-0 transition hover:bg-zinc-800 hover:text-zinc-200 group-hover:opacity-100"
          >
            ✎
          </button>
          <span
            className={`h-2.5 w-2.5 shrink-0 rounded-full ${color} ${
              online ? "animate-pulse" : ""
            }`}
            title={online ? "在线" : "离线"}
          />
        </div>
      </div>

      {metrics ? (
        <div className="flex flex-col gap-2.5">
          <MetricRow
            label="CPU"
            text={`${metrics.cpu_percent.toFixed(1)}%`}
            percent={metrics.cpu_percent}
            color={metrics.cpu_percent > 80 ? "bg-red-500" : "bg-indigo-500"}
          />
          <MetricRow
            label="内存"
            text={`${metrics.mem_used_mb}/${metrics.mem_total_mb} MB`}
            percent={metrics.mem_percent}
            color={metrics.mem_percent > 80 ? "bg-red-500" : "bg-emerald-500"}
          />
          <MetricRow
            label="磁盘"
            text={`${metrics.disk_used_gb.toFixed(1)}/${metrics.disk_total_gb.toFixed(1)} G`}
            percent={metrics.disk_percent}
            color={metrics.disk_percent > 85 ? "bg-red-500" : "bg-amber-500"}
          />
          <div className="flex items-center justify-between border-t border-zinc-800 pt-2 text-xs">
            <span className="text-zinc-500">负载</span>
            <span className="font-mono text-zinc-400">
              {metrics.load_1.toFixed(2)} / {metrics.load_5.toFixed(2)} /{" "}
              {metrics.load_15.toFixed(2)}
            </span>
          </div>
        </div>
      ) : (
        <div className="flex h-32 items-center justify-center text-xs text-zinc-600">
          {initializing || !fetchedOnce ? "加载中..." : "无法连接 / 凭证缺失"}
        </div>
      )}
    </div>
  );
}

export default function Dashboard() {
  const { hosts, loading, load } = useHostStore();
  const [modalOpen, setModalOpen] = useState(false);
  const [editId, setEditId] = useState<string | null>(null);

  useEffect(() => {
    load();
  }, [load]);

  const hostIds = useMemo(() => hosts.map((h) => h.id), [hosts]);
  const editingHost = editId ? hosts.find((h) => h.id === editId) : undefined;

  function openAdd() {
    setEditId(null);
    setModalOpen(true);
  }

  function openEdit(id: string) {
    setEditId(id);
    setModalOpen(true);
  }

  return (
    <div className="p-6">
      <div className="mb-6 flex items-center justify-between">
        <h1 className="text-xl font-semibold">总览</h1>
        <div className="flex items-center gap-3">
          <span className="text-xs text-zinc-500">
            前台 1s / 后台 10s 自动刷新 · {hosts.length} 台主机
          </span>
          <button
            onClick={openAdd}
            className="rounded-md bg-indigo-500 px-4 py-2 text-sm font-medium text-white transition hover:bg-indigo-400"
          >
            + 添加主机
          </button>
        </div>
      </div>

      {loading && <p className="text-sm text-zinc-500">加载中...</p>}
      <div className="grid grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-3">
        {hostIds.map((id) => (
          <HostCard key={id} id={id} onEdit={openEdit} />
        ))}
        {!loading && hosts.length === 0 && (
          <div className="rounded-lg border border-dashed border-zinc-700 p-10 text-center text-sm text-zinc-500">
            还没有主机，点击右上角「添加主机」开始
          </div>
        )}
      </div>

      {modalOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-6">
          <div className="max-h-[90vh] w-full max-w-xl overflow-auto rounded-lg border border-zinc-700 bg-zinc-950">
            <HostForm
              editing={editingHost}
              onCancel={() => setModalOpen(false)}
              onSaved={() => {
                setModalOpen(false);
                load();
              }}
            />
          </div>
        </div>
      )}
    </div>
  );
}
