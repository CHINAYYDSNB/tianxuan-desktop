import { useEffect, useState } from "react";

import { useHostStore } from "../../stores/hostStore";
import {
  batchExec,
  listCommandHistory,
  type BatchResult,
  type CommandHistory,
} from "../../lib/tauri";

function ResultPanel({ result }: { result: BatchResult }) {
  const [open, setOpen] = useState(true);
  return (
    <div className="overflow-hidden rounded-lg border border-zinc-800 bg-zinc-950">
      <div
        className="flex cursor-pointer items-center gap-3 px-3 py-2"
        onClick={() => setOpen(!open)}
      >
        <span className={`text-xs ${result.success ? "text-green-400" : "text-red-400"}`}>
          {result.success ? "✓" : "✗"}
        </span>
        <span className="font-medium text-sm">{result.host_name}</span>
        <span className="ml-auto text-xs text-zinc-500">
          exit {result.exit_code ?? "-"} · {result.elapsed_ms}ms
        </span>
        <span className="text-xs text-zinc-600">{open ? "▾" : "▸"}</span>
      </div>
      {open && (
        <div className="border-t border-zinc-800">
          {result.error && (
            <pre className="max-h-64 overflow-auto bg-red-500/5 px-3 py-2 font-mono text-xs text-red-400">
              {result.error}
            </pre>
          )}
          {result.stdout && (
            <pre className="max-h-64 overflow-auto px-3 py-2 font-mono text-xs text-zinc-200">
              {result.stdout}
            </pre>
          )}
          {result.stderr && (
            <pre className="max-h-48 overflow-auto border-t border-zinc-800 px-3 py-2 font-mono text-xs text-amber-400">
              {result.stderr}
            </pre>
          )}
          {!result.stdout && !result.stderr && !result.error && (
            <div className="px-3 py-2 text-xs text-zinc-600">无输出</div>
          )}
        </div>
      )}
    </div>
  );
}

export default function BatchCommand() {
  const { hosts, load } = useHostStore();
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [command, setCommand] = useState("");
  const [running, setRunning] = useState(false);
  const [results, setResults] = useState<BatchResult[] | null>(null);
  const [history, setHistory] = useState<CommandHistory[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    load();
    listCommandHistory()
      .then(setHistory)
      .catch(() => {});
  }, [load]);

  function toggle(id: string) {
    const next = new Set(selected);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    setSelected(next);
  }

  function toggleAll() {
    if (selected.size === hosts.length) setSelected(new Set());
    else setSelected(new Set(hosts.map((h) => h.id)));
  }

  async function run() {
    if (!command.trim() || selected.size === 0) return;
    setRunning(true);
    setError(null);
    setResults(null);
    try {
      const res = await batchExec(Array.from(selected), command);
      setResults(res);
      const hist = await listCommandHistory();
      setHistory(hist);
    } catch (e) {
      setError(String(e));
    } finally {
      setRunning(false);
    }
  }

  function replay(cmd: string) {
    setCommand(cmd);
  }

  return (
    <div className="flex h-full flex-col">
      <div className="flex items-center gap-3 border-b border-zinc-800 p-3">
        <h1 className="text-base font-semibold">批量命令</h1>
        <span className="text-xs text-zinc-500">已选 {selected.size} 台</span>
        {running && (
          <span className="flex items-center gap-1 text-xs text-indigo-400">
            <span className="h-2 w-2 animate-ping rounded-full bg-indigo-400" />
            执行中...
          </span>
        )}
      </div>

      <div className="grid min-h-0 flex-1 grid-cols-3 gap-4 p-4">
        {/* host selection */}
        <div className="flex flex-col overflow-hidden rounded-lg border border-zinc-800 bg-zinc-950">
          <div className="flex items-center justify-between border-b border-zinc-800 px-3 py-2">
            <span className="text-sm font-medium">主机</span>
            <button
              onClick={toggleAll}
              className="text-xs text-indigo-400 hover:underline"
            >
              {selected.size === hosts.length && hosts.length > 0
                ? "全不选"
                : "全选"}
            </button>
          </div>
          <div className="min-h-0 flex-1 overflow-auto p-2">
            {hosts.map((h) => (
              <label
                key={h.id}
                className="flex cursor-pointer items-center gap-2 rounded px-2 py-1.5 text-sm hover:bg-zinc-800/60"
              >
                <input
                  type="checkbox"
                  checked={selected.has(h.id)}
                  onChange={() => toggle(h.id)}
                  className="accent-indigo-500"
                />
                <span className="flex-1 truncate">{h.name}</span>
                <span className="text-xs text-zinc-600">{h.address}</span>
              </label>
            ))}
            {hosts.length === 0 && (
              <p className="p-3 text-xs text-zinc-600">暂无主机</p>
            )}
          </div>
        </div>

        {/* command + results */}
        <div className="col-span-2 flex min-h-0 flex-col gap-3">
          <div className="flex gap-2">
            <input
              value={command}
              onChange={(e) => setCommand(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && run()}
              placeholder="输入要批量执行的命令，如：uptime"
              className="flex-1 rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 font-mono text-sm outline-none focus:border-indigo-500"
            />
            <button
              onClick={run}
              disabled={running || selected.size === 0 || !command.trim()}
              className="rounded-md bg-indigo-500 px-4 py-2 text-sm font-medium text-white transition hover:bg-indigo-400 disabled:opacity-40"
            >
              执行
            </button>
          </div>

          {error && <p className="text-sm text-red-400">{error}</p>}

          <div className="min-h-0 flex-1 space-y-2 overflow-auto">
            {results === null && (
              <div className="flex h-full items-center justify-center text-sm text-zinc-600">
                选择主机并输入命令后点击「执行」
              </div>
            )}
            {results?.map((r) => (
              <ResultPanel key={r.host_id} result={r} />
            ))}
          </div>
        </div>
      </div>

      {/* history */}
      {history.length > 0 && (
        <div className="border-t border-zinc-800 p-3">
          <div className="mb-2 text-xs font-medium text-zinc-500">
            最近命令
          </div>
          <div className="flex flex-wrap gap-2">
            {history.slice(0, 10).map((h) => (
              <button
                key={h.id}
                onClick={() => replay(h.command)}
                className="rounded border border-zinc-700 px-2 py-1 font-mono text-xs text-zinc-300 hover:border-indigo-500"
                title={`${h.host_count} 台 · 成功 ${h.success_count} 失败 ${h.fail_count}`}
              >
                {h.command}
              </button>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
