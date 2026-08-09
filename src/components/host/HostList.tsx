import { useEffect, useMemo } from "react";
import { Link } from "react-router-dom";

import { useHostStore } from "../../stores/hostStore";
import type { Host } from "../../lib/tauri";

function HostRow({ host }: { host: Host }) {
  return (
    <div className="flex items-center gap-4 rounded-lg border border-zinc-800 bg-zinc-950 p-3">
      <div className="flex h-9 w-9 items-center justify-center rounded-md bg-zinc-800 text-sm">
        🖥
      </div>
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          <span className="font-medium">{host.name}</span>
          {host.tags.map((t) => (
            <span
              key={t}
              className="rounded bg-indigo-500/15 px-1.5 py-0.5 text-[10px] text-indigo-300"
            >
              {t}
            </span>
          ))}
        </div>
        <div className="text-xs text-zinc-500">
          {host.username}@{host.address}:{host.port}
        </div>
      </div>
      <div className="flex gap-2">
        <Link
          to={`/hosts/${host.id}/edit`}
          className="rounded-md border border-zinc-700 px-3 py-1.5 text-xs text-zinc-300 transition hover:border-indigo-500"
        >
          编辑
        </Link>
      </div>
    </div>
  );
}

export default function HostList() {
  const { hosts, loading, error, load } = useHostStore();

  useEffect(() => {
    load();
  }, [load]);

  const groups = useMemo(() => {
    const map = new Map<string, Host[]>();
    for (const h of hosts) {
      const list = map.get(h.group_name) ?? [];
      list.push(h);
      map.set(h.group_name, list);
    }
    return Array.from(map.entries());
  }, [hosts]);

  return (
    <div className="p-6">
      <div className="mb-6 flex items-center justify-between">
        <h1 className="text-xl font-semibold">主机管理</h1>
        <Link
          to="/hosts/new"
          className="rounded-md bg-indigo-500 px-4 py-2 text-sm font-medium text-white transition hover:bg-indigo-400"
        >
          + 添加主机
        </Link>
      </div>

      {error && <p className="mb-4 text-sm text-red-400">{error}</p>}
      {loading && <p className="text-sm text-zinc-500">加载中...</p>}

      {!loading && hosts.length === 0 && (
        <div className="rounded-lg border border-dashed border-zinc-700 p-10 text-center text-sm text-zinc-500">
          还没有主机，点击右上角「添加主机」开始
        </div>
      )}

      <div className="flex flex-col gap-6">
        {groups.map(([group, list]) => (
          <div key={group}>
            <h2 className="mb-2 text-sm font-medium text-zinc-400">{group}</h2>
            <div className="flex flex-col gap-2">
              {list.map((h) => (
                <HostRow key={h.id} host={h} />
              ))}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
