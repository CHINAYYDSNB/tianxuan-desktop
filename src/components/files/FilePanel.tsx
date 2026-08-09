import { useCallback, useEffect, useRef, useState } from "react";

import {
  sftpDelete,
  sftpDownload,
  sftpList,
  sftpReadText,
  sftpRename,
  sftpUpload,
  sftpWriteText,
  type FileEntry,
} from "../../lib/tauri";

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024)
    return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
  return `${(bytes / 1024 / 1024 / 1024).toFixed(2)} GB`;
}

interface FilePanelProps {
  hostId: string;
}

export default function FilePanel({ hostId }: FilePanelProps) {
  const [path, setPath] = useState("/root");
  const [entries, setEntries] = useState<FileEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [editing, setEditing] = useState<FileEntry | null>(null);
  const [editContent, setEditContent] = useState("");
  const [renameTarget, setRenameTarget] = useState<FileEntry | null>(null);
  const [renameValue, setRenameValue] = useState("");
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [uploading, setUploading] = useState(false);

  const refresh = useCallback(async () => {
    if (!hostId) return;
    setLoading(true);
    setError(null);
    try {
      const items = await sftpList(hostId, path);
      setEntries(items);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, [hostId, path]);

  useEffect(() => {
    if (hostId) refresh();
  }, [hostId, path, refresh]);

  function goTo(dir: string) {
    setPath(dir);
  }

  function goUp() {
    if (path === "/" || path === "") return;
    const idx = path.lastIndexOf("/");
    setPath(idx <= 0 ? "/" : path.slice(0, idx));
  }

  async function handleUpload(file: File) {
    if (!hostId || !file) return;
    setUploading(true);
    setError(null);
    try {
      await sftpUpload(hostId, file.name, `${path}/${file.name}`);
      await refresh();
    } catch (e) {
      setError(String(e));
    } finally {
      setUploading(false);
    }
  }

  async function handleDownload(entry: FileEntry) {
    if (!hostId) return;
    setError(null);
    try {
      const local = `C:\\Users\\Administrator\\Downloads\\${entry.name}`;
      await sftpDownload(hostId, entry.path, local);
      alert(`已下载到 ${local}`);
    } catch (e) {
      setError(String(e));
    }
  }

  async function handleDelete(entry: FileEntry) {
    if (!hostId) return;
    if (!confirm(`确认删除 ${entry.path}？`)) return;
    setError(null);
    try {
      await sftpDelete(hostId, entry.path);
      await refresh();
    } catch (e) {
      setError(String(e));
    }
  }

  async function openEditor(entry: FileEntry) {
    if (!hostId) return;
    setError(null);
    try {
      const content = await sftpReadText(hostId, entry.path);
      setEditing(entry);
      setEditContent(content);
    } catch (e) {
      setError(String(e));
    }
  }

  async function saveEditor() {
    if (!hostId || !editing) return;
    setError(null);
    try {
      await sftpWriteText(hostId, editing.path, editContent);
      setEditing(null);
      await refresh();
    } catch (e) {
      setError(String(e));
    }
  }

  async function confirmRename() {
    if (!hostId || !renameTarget || !renameValue) return;
    setError(null);
    try {
      const newPath = `${renameTarget.path.slice(0, renameTarget.path.lastIndexOf("/") + 1)}${renameValue}`;
      await sftpRename(hostId, renameTarget.path, newPath);
      setRenameTarget(null);
      await refresh();
    } catch (e) {
      setError(String(e));
    }
  }

  return (
    <div className="flex h-full flex-col">
      <div className="flex items-center gap-2 border-b border-zinc-800 p-2">
        <button
          onClick={goUp}
          disabled={path === "/"}
          className="rounded-md border border-zinc-700 px-2 py-1 text-xs text-zinc-300 transition hover:border-indigo-500 disabled:opacity-40"
        >
          ⬆
        </button>
        <div className="flex min-w-0 flex-1 items-center gap-1 rounded-md border border-zinc-800 bg-zinc-950 px-2 py-1 text-xs text-zinc-300">
          <input
            value={path}
            onChange={(e) => setPath(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && refresh()}
            className="w-full bg-transparent outline-none"
          />
        </div>
        <button
          onClick={refresh}
          className="rounded-md bg-indigo-500 px-2 py-1 text-xs font-medium text-white transition hover:bg-indigo-400"
        >
          刷新
        </button>
        <button
          onClick={() => fileInputRef.current?.click()}
          disabled={uploading}
          className="rounded-md border border-zinc-600 px-2 py-1 text-xs text-zinc-300 transition hover:border-indigo-500 disabled:opacity-40"
        >
          {uploading ? "上传中..." : "上传"}
        </button>
        <input
          ref={fileInputRef}
          type="file"
          className="hidden"
          onChange={(e) => {
            const f = e.target.files?.[0];
            if (f) handleUpload(f);
            e.target.value = "";
          }}
        />
      </div>

      {error && <p className="px-2 py-1 text-xs text-red-400">{error}</p>}

      <div className="min-h-0 flex-1 overflow-auto">
        <table className="w-full text-xs">
          <tbody>
            {loading && (
              <tr>
                <td className="px-2 py-2 text-zinc-600">加载中...</td>
              </tr>
            )}
            {!loading && entries.length === 0 && (
              <tr>
                <td className="px-2 py-2 text-zinc-600">空目录</td>
              </tr>
            )}
            {entries.map((entry) => (
              <tr
                key={entry.path}
                className="border-t border-zinc-800/60 hover:bg-zinc-800/40"
              >
                <td
                  className="cursor-pointer truncate px-2 py-1.5"
                  onClick={() => entry.is_dir && goTo(entry.path)}
                >
                  <span className="mr-1">{entry.is_dir ? "📁" : "📄"}</span>
                  <span className="truncate">{entry.name}</span>
                </td>
                <td className="px-1 py-1.5 text-right font-mono text-[10px] text-zinc-500">
                  {entry.is_dir ? "" : formatSize(entry.size)}
                </td>
                <td className="px-1 py-1.5">
                  <div className="flex justify-end gap-1">
                    {!entry.is_dir && (
                      <>
                        <button
                          onClick={() => openEditor(entry)}
                          className="rounded border border-zinc-700 px-1.5 py-0.5 text-[10px] text-zinc-300 hover:border-indigo-500"
                        >
                          编辑
                        </button>
                        <button
                          onClick={() => handleDownload(entry)}
                          className="rounded border border-zinc-700 px-1.5 py-0.5 text-[10px] text-zinc-300 hover:border-indigo-500"
                        >
                          下载
                        </button>
                      </>
                    )}
                    <button
                      onClick={() => {
                        setRenameTarget(entry);
                        setRenameValue(entry.name);
                      }}
                      className="rounded border border-zinc-700 px-1.5 py-0.5 text-[10px] text-zinc-300 hover:border-indigo-500"
                    >
                      改名
                    </button>
                    <button
                      onClick={() => handleDelete(entry)}
                      className="rounded border border-red-500/30 px-1.5 py-0.5 text-[10px] text-red-400 hover:bg-red-500/10"
                    >
                      删
                    </button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {editing && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-6">
          <div className="flex h-[80vh] w-[90vw] flex-col overflow-hidden rounded-lg border border-zinc-700 bg-zinc-950">
            <div className="flex items-center justify-between border-b border-zinc-800 px-4 py-2">
              <span className="truncate font-mono text-sm text-zinc-300">
                编辑 {editing.path}
              </span>
              <div className="flex gap-2">
                <button
                  onClick={saveEditor}
                  className="rounded-md bg-indigo-500 px-3 py-1 text-sm font-medium text-white hover:bg-indigo-400"
                >
                  保存
                </button>
                <button
                  onClick={() => setEditing(null)}
                  className="rounded-md border border-zinc-600 px-3 py-1 text-sm text-zinc-300"
                >
                  取消
                </button>
              </div>
            </div>
            <textarea
              value={editContent}
              onChange={(e) => setEditContent(e.target.value)}
              className="flex-1 bg-zinc-950 p-4 font-mono text-sm text-zinc-200 outline-none"
              spellCheck={false}
            />
          </div>
        </div>
      )}

      {renameTarget && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-6">
          <div className="w-80 rounded-lg border border-zinc-700 bg-zinc-950 p-4">
            <h3 className="mb-3 text-sm font-medium">重命名</h3>
            <input
              value={renameValue}
              onChange={(e) => setRenameValue(e.target.value)}
              className="mb-4 w-full rounded-md border border-zinc-700 bg-zinc-900 px-3 py-2 text-sm outline-none focus:border-indigo-500"
              autoFocus
            />
            <div className="flex justify-end gap-2">
              <button
                onClick={() => setRenameTarget(null)}
                className="rounded-md border border-zinc-600 px-3 py-1.5 text-sm text-zinc-300"
              >
                取消
              </button>
              <button
                onClick={confirmRename}
                className="rounded-md bg-indigo-500 px-3 py-1.5 text-sm text-white hover:bg-indigo-400"
              >
                确认
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
