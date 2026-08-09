import { useEffect, useRef, useState } from "react";
import { Terminal as XTerm } from "xterm";
import { FitAddon } from "@xterm/addon-fit";
import "xterm/css/xterm.css";

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

interface TerminalViewProps {
  hostId: string;
  hostLabel?: string;
}

export default function TerminalView({ hostId, hostLabel }: TerminalViewProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const termRef = useRef<XTerm | null>(null);
  const fitRef = useRef<FitAddon | null>(null);
  const sessionIdRef = useRef<string>("");
  const [connected, setConnected] = useState(false);
  const [connecting, setConnecting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const hostIdRef = useRef(hostId);

  useEffect(() => {
    hostIdRef.current = hostId;
  }, [hostId]);

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

  async function connect(force = false) {
    const hid = hostIdRef.current;
    if (!hid || (connected && !force)) return;
    setConnecting(true);
    setError(null);
    if (sessionIdRef.current) {
      await sshCloseSession(sessionIdRef.current).catch(() => {});
    }
    const sid = newSessionId();
    sessionIdRef.current = sid;
    try {
      await sshOpenSession(hid, sid);
      setConnected(true);
      termRef.current?.writeln("\r\n\x1b[32m[已连接]\x1b[0m");
      if (fitRef.current && termRef.current) {
        const cols = termRef.current.cols;
        const rows = termRef.current.rows;
        sshResize(sid, cols, rows).catch(() => {});
      }
    } catch (e) {
      setError(String(e));
      sessionIdRef.current = "";
    } finally {
      setConnecting(false);
    }
  }

  function disconnect() {
    if (sessionIdRef.current) {
      sshCloseSession(sessionIdRef.current).catch(() => {});
      sessionIdRef.current = "";
      setConnected(false);
    }
  }

  // auto-connect when hostId changes
  useEffect(() => {
    if (hostId) {
      const t = setTimeout(() => connect(true), 200);
      return () => clearTimeout(t);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [hostId]);

  return (
    <div className="flex h-full flex-col">
      <div className="flex items-center gap-3 border-b border-zinc-800 p-2">
        <span className="text-xs text-zinc-500">
          {hostLabel ?? hostId}
        </span>
        <button
          onClick={() => connect(true)}
          disabled={connecting}
          className="rounded-md border border-zinc-600 px-3 py-1 text-xs text-zinc-300 transition hover:border-indigo-500 disabled:opacity-40"
        >
          {connecting ? "连接中..." : connected ? "重连" : "连接"}
        </button>
        <button
          onClick={disconnect}
          disabled={!connected}
          className="rounded-md border border-zinc-600 px-3 py-1 text-xs text-zinc-300 transition hover:border-red-500 disabled:opacity-40"
        >
          断开
        </button>
        {connected && (
          <span className="ml-auto flex items-center gap-1 text-xs text-green-400">
            <span className="h-2 w-2 rounded-full bg-green-400" /> 已连接
          </span>
        )}
      </div>
      {error && <p className="px-2 py-1 text-xs text-red-400">{error}</p>}
      <div className="min-h-0 flex-1 bg-[#0d1117] p-1">
        <div ref={containerRef} className="h-full" />
      </div>
    </div>
  );
}
