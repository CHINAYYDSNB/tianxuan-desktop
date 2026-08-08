import { useEffect } from "react";
import { Link } from "react-router-dom";

import { useHostStore } from "../../stores/hostStore";

export default function Dashboard() {
  const { hosts, loading, load } = useHostStore();

  useEffect(() => {
    load();
  }, [load]);

  return (
    <div className="p-6">
      <h1 className="mb-6 text-xl font-semibold">总览</h1>
      {loading && <p className="text-sm text-zinc-500">加载中...</p>}
      <div className="grid grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-3">
        {hosts.map((h) => (
          <div
            key={h.id}
            className="rounded-lg border border-zinc-800 bg-zinc-950 p-4"
          >
            <div className="mb-2 flex items-center justify-between">
              <span className="font-medium">{h.name}</span>
              <span className="h-2 w-2 rounded-full bg-zinc-600" />
            </div>
            <div className="text-xs text-zinc-500">
              {h.address}:{h.port}
            </div>
            <div className="mt-3 flex gap-2 text-xs">
              <Link
                to={`/hosts/${h.id}/edit`}
                className="rounded border border-zinc-700 px-2 py-1 text-zinc-300 hover:border-indigo-500"
              >
                详情
              </Link>
            </div>
          </div>
        ))}
        {!loading && hosts.length === 0 && (
          <p className="text-sm text-zinc-500">暂无主机</p>
        )}
      </div>
    </div>
  );
}
