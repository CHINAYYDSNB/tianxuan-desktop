import { useEffect, useRef, useState } from "react";
import { Terminal as XTerm } from "xterm";
import { FitAddon } from "@xterm/addon-fit";
import "xterm/css/xterm.css";

import { useHostStore } from "../../stores/hostStore";
import {
  listenTerminalOutput,
  sshCloseSession,
  sshOpenSession,
  sshResize,
  sshWrite,
} from "../../lib/tauri";

function newSessionId(): string {
  if (typeof crypto !== "undefined" && crypto.randomUUID) {
    return crypto.randomUUID();
  }
  return `${Date.now()}-${Math.random().toString(36).slice(2)}`;
}

export default function TerminalPage() {
  const { hosts, load } = useHostStore();
  const [selectedHostId, setSelectedHostId] = useState<string>("");
  const containerRef = useRef<HTMLDivElement>(null);
  const termRef = useRef<XTerm | null>(null);
  const fitRef = useRef<FitAddon | null>(null);
  const sessionIdRef = useRef<string>("");
  const [connected, setConnected] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    load();
  }, [load]);

  // init xterm once
  useEffect(() => {
    if (!containerRef.current) return;
    const term = new XTerm({
      cursorBlink: true,
      fontFamily: "JetBrains Mono, Consolas, monospace",
      fontSize: 14,
      theme: {
        background: "#0d1117",
        foreground: "#e6edf3",
      },
    });
    const fit = new FitAddon();
    term.loadAddon(fit);
    term.open(containerRef.current);
    fit.fit();
    termRef.current = term;
    fitRef.current = fit;

    term.onData((data) => {
      if (sessionIdRef.current) {
        const bytes = new TextEncoder().encode(data);
        sshWrite(sessionIdRef.current, bytes).catch(() => {});
      }
    });

    const unlistenPromise = listenTerminalOutput((p) => {
      if (p.session_id === sessionIdRef.current && termRef.current) {
        if (p.data === "\u0000[EXIT]") {
          termRef.current.writeln("\r\n[进程已退出]");
          setConnected(false);
        } else {
          termRef.current.write(p.data);
        }
      }
    });

    return () => {
      if (sessionIdRef.current) {
        sshCloseSession(sessionIdRef.current).catch(() => {});
      }
      unlistenPromise.then((un) => un());
      term.dispose();
    };
  }, []);

  async function connect() {
    if (!selectedHostId) return;
    setError(null);
    setConnected(false);
    if (sessionIdRef.current) {
      await sshCloseSession(sessionIdRef.current).catch(() => {});
    }
    const sid = newSessionId();
    sessionIdRef.current = sid;
    try {
      await sshOpenSession(selectedHostId, sid);
      setConnected(true);
      termRef.current?.writeln("\r\n\x1b[32m[已连接]\x1b[0m");
      if (fitRef.current) {
        const dims = fitRef.current.proposeDimensions();
        if (dims && termRef.current) {
          const cols = termRef.current.cols;
          const rows = termRef.current.rows;
          sshResize(sid, cols, rows).catch(() => {});
        }
      }
    } catch (e) {
      setError(String(e));
    }
  }

  function disconnect() {
    if (sessionIdRef.current) {
      sshCloseSession(sessionIdRef.current).catch(() => {});
      sessionIdRef.current = "";
      setConnected(false);
    }
  }

  const selectedHost = hosts.find((h) => h.id === selectedHostId);

  return (
    <div className="flex h-full flex-col">
      <div className="flex items-center gap-3 border-b border-zinc-800 p-3">
        <select
          className="rounded-md border border-zinc-700 bg-zinc-950 px-3 py-1.5 text-sm outline-none focus:border-indigo-500"
          value={selectedHostId}
          onChange={(e) => setSelectedHostId(e.target.value)}
        >
          <option value="">选择主机...</option>
          {hosts.map((h) => (
            <option key={h.id} value={h.id}>
              {h.name} ({h.address})
            </option>
          ))}
        </select>
        <button
          onClick={connect}
          disabled={!selectedHostId || connected}
          className="rounded-md bg-indigo-500 px-3 py-1.5 text-sm font-medium text-white transition hover:bg-indigo-400 disabled:opacity-40"
        >
          连接
        </button>
        <button
          onClick={disconnect}
          disabled={!connected}
          className="rounded-md border border-zinc-600 px-3 py-1.5 text-sm text-zinc-300 transition hover:border-red-500 disabled:opacity-40"
        >
          断开
        </button>
        {selectedHost && (
          <span className="text-xs text-zinc-500">
            {selectedHost.name}@{selectedHost.address}
          </span>
        )}
        {connected && (
          <span className="ml-auto flex items-center gap-1 text-xs text-green-400">
            <span className="h-2 w-2 rounded-full bg-green-400" /> 已连接
          </span>
        )}
      </div>
      {error && <p className="px-3 py-2 text-sm text-red-400">{error}</p>}
      <div className="min-h-0 flex-1 bg-[#0d1117] p-2">
        <div ref={containerRef} className="h-full" />
      </div>
    </div>
  );
}
