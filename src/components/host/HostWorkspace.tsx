import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";

import { useHostStore } from "../../stores/hostStore";
import TerminalView from "../terminal/TerminalView";
import MonitorPanel from "../monitor/MonitorPanel";
import FilePanel from "../files/FilePanel";

type SideTab = "monitor" | "files";

export default function HostWorkspace() {
  const { id } = useParams();
  const { hosts, load } = useHostStore();
  const [tab, setTab] = useState<SideTab>("monitor");

  useEffect(() => {
    load();
  }, [load]);

  const host = hosts.find((h) => h.id === id);

  if (!host) {
    return (
      <div className="flex h-full items-center justify-center p-6 text-sm text-zinc-500">
        主机不存在或未加载
      </div>
    );
  }

  return (
    <div className="flex h-full">
      <div className="min-w-0 flex-1">
        <TerminalView hostId={host.id} hostLabel={`${host.name} (${host.address})`} />
      </div>

      <aside className="flex w-80 shrink-0 flex-col border-l border-zinc-800 bg-zinc-950">
        <div className="flex border-b border-zinc-800">
          {(
            [
              ["monitor", "监控"],
              ["files", "文件"],
            ] as [SideTab, string][]
          ).map(([key, label]) => (
            <button
              key={key}
              onClick={() => setTab(key)}
              className={`flex-1 px-3 py-2 text-sm transition ${
                tab === key
                  ? "border-b-2 border-indigo-500 text-indigo-300"
                  : "text-zinc-500 hover:text-zinc-300"
              }`}
            >
              {label}
            </button>
          ))}
        </div>
        <div className="min-h-0 flex-1 overflow-auto">
          {tab === "monitor" ? (
            <MonitorPanel hostId={host.id} />
          ) : (
            <FilePanel hostId={host.id} />
          )}
        </div>
      </aside>
    </div>
  );
}
