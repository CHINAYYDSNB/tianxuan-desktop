import { useState } from "react";

import { collectMetrics, type HostMetrics } from "../../lib/tauri";
import { useVisibilityPolling } from "../../hooks/useVisibilityPolling";

function Bar({ value, color }: { value: number; color: string }) {
  const clamped = Math.max(0, Math.min(100, value));
  return (
    <div className="h-1 w-full overflow-hidden rounded-full bg-zinc-800">
      <div
        className={`h-full rounded-full ${color} transition-all duration-700`}
        style={{ width: `${clamped}%` }}
      />
    </div>
  );
}

function Row({
  label,
  text,
  percent,
  color,
}: {
  label: string;
  text: string;
  percent?: number;
  color?: string;
}) {
  return (
    <div className="flex flex-col gap-0.5">
      <div className="flex items-center justify-between text-xs">
        <span className="text-zinc-500">{label}</span>
        <span className="font-mono text-zinc-300">{text}</span>
      </div>
      {percent !== undefined && color && <Bar value={percent} color={color} />}
    </div>
  );
}

interface MonitorPanelProps {
  hostId: string;
}

export default function MonitorPanel({ hostId }: MonitorPanelProps) {
  const [metrics, setMetrics] = useState<HostMetrics | null>(null);

  useVisibilityPolling(async () => {
    try {
      const m = await collectMetrics(hostId);
      setMetrics(m);
    } catch {
      // keep previous metrics
    }
  });

  if (!metrics) {
    return (
      <div className="flex h-full items-center justify-center p-4 text-xs text-zinc-600">
        采集指标中...
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-3 p-3 text-xs">
      <div className="flex items-center justify-between">
        <span className="flex items-center gap-1 text-zinc-400">
          <span
            className={`h-2 w-2 rounded-full ${metrics.online ? "bg-emerald-400" : "bg-zinc-600"}`}
          />
          {metrics.online ? "在线" : "离线"}
        </span>
        <span className="font-mono text-zinc-500">1s 刷新</span>
      </div>

      <div className="flex flex-col gap-2.5">
        <Row
          label="CPU"
          text={`${metrics.cpu_percent.toFixed(1)}%`}
          percent={metrics.cpu_percent}
          color={metrics.cpu_percent > 80 ? "bg-red-500" : "bg-indigo-500"}
        />
        <Row
          label="内存"
          text={`${metrics.mem_used_mb}/${metrics.mem_total_mb} MB (${metrics.mem_percent.toFixed(0)}%)`}
          percent={metrics.mem_percent}
          color={metrics.mem_percent > 80 ? "bg-red-500" : "bg-emerald-500"}
        />
        <Row
          label="磁盘"
          text={`${metrics.disk_used_gb.toFixed(1)}/${metrics.disk_total_gb.toFixed(1)} G (${metrics.disk_percent.toFixed(0)}%)`}
          percent={metrics.disk_percent}
          color={metrics.disk_percent > 85 ? "bg-red-500" : "bg-amber-500"}
        />
        <div className="flex flex-col gap-0.5">
          <div className="flex items-center justify-between">
            <span className="text-zinc-500">负载</span>
            <span className="font-mono text-zinc-300">
              {metrics.load_1.toFixed(2)} / {metrics.load_5.toFixed(2)} /{" "}
              {metrics.load_15.toFixed(2)}
            </span>
          </div>
        </div>
      </div>

      <div className="border-t border-zinc-800 pt-2">
        <div className="mb-1 text-zinc-500">IO 吞吐</div>
        <Row
          label="读"
          text={`${metrics.io_read_kbps.toFixed(1)} KB/s`}
        />
        <Row
          label="写"
          text={`${metrics.io_write_kbps.toFixed(1)} KB/s`}
        />
      </div>

      <div className="border-t border-zinc-800 pt-2">
        <div className="mb-1 text-zinc-500">网络吞吐</div>
        <Row
          label="下行"
          text={`${metrics.net_rx_kbps.toFixed(1)} KB/s`}
        />
        <Row
          label="上行"
          text={`${metrics.net_tx_kbps.toFixed(1)} KB/s`}
        />
      </div>
    </div>
  );
}
