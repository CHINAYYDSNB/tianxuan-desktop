import { useCallback, useEffect, useRef, useState } from "react";

import { useHostStore } from "../../stores/hostStore";
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

function permString(mode: number): string {
  const chars = ["---", "--x", "-w-", "-wx", "r--", "r-x", "rw-", "rwx"];
  return (
    (mode & 0o400 ? "d" : "-") +
    chars[(mode >> 6) & 7] +
    chars[(mode >> 3) & 7] +
    chars[mode & 7]
  );
}

export default function FileManager() {
  const { hosts, load } = useHostStore();
  const [selectedHostId, setSelectedHostId] = useState("");
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

  useEffect(() => {
    load();
  }, [load]);

  const refresh = useCallback(async () => {
    if (!selectedHostId) return;
    setLoading(true);
    setError(null);
    try {
      const items = await sftpList(selectedHostId, path);
      setEntries(items);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, [selectedHostId, path]);

  useEffect(() => {
    if (selectedHostId) refresh();
  }, [selectedHostId, path, refresh]);

  function goTo(dir: string) {
    setPath(dir);
  }

  function goUp() {
    if (path === "/" || path === "") return;
    const idx = path.lastIndexOf("/");
    setPath(idx <= 0 ? "/" : path.slice(0, idx));
  }

  async function handleUpload(file: File) {
    if (!selectedHostId || !file) return;
    setUploading(true);
    setError(null);
    try {
      await sftpUpload(selectedHostId, file.name, `${path}/${file.name}`);
      await refresh();
    } catch (e) {
      setError(String(e));
    } finally {
      setUploading(false);
    }
  }

  async function handleDownload(entry: FileEntry) {
    if (!selectedHostId) return;
    setError(null);
    try {
      const local = `C:\\Users\\Administrator\\Downloads\\${entry.name}`;
      await sftpDownload(selectedHostId, entry.path, local);
      alert(`已下载到 ${local}`);
    } catch (e) {
      setError(String(e));
    }
  }

  async function handleDelete(entry: FileEntry) {
    if (!selectedHostId) return;
    if (!confirm(`确认删除 ${entry.path}？`)) return;
    setError(null);
    try {
      await sftpDelete(selectedHostId, entry.path);
      await refresh();
    } catch (e) {
      setError(String(e));
    }
  }

  async function openEditor(entry: FileEntry) {
    if (!selectedHostId) return;
    setError(null);
    try {
      const content = await sftpReadText(selectedHostId, entry.path);
      setEditing(entry);
      setEditContent(content);
    } catch (e) {
      setError(String(e));
    }
  }

  async function saveEditor() {
    if (!selectedHostId || !editing) return;
    setError(null);
    try {
      await sftpWriteText(selectedHostId, editing.path, editContent);
      setEditing(null);
      await refresh();
    } catch (e) {
      setError(String(e));
    }
  }

  async function confirmRename() {
    if (!selectedHostId || !renameTarget || !renameValue) return;
    setError(null);
    try {
      const newPath = `${renameTarget.path.slice(0, renameTarget.path.lastIndexOf("/") + 1)}${renameValue}`;
      await sftpRename(selectedHostId, renameTarget.path, newPath);
      setRenameTarget(null);
      await refresh();
    } catch (e) {
      setError(String(e));
    }
  }

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
          onClick={goUp}
          disabled={!selectedHostId || path === "/"}
          className="rounded-md border border-zinc-700 px-3 py-1.5 text-sm text-zinc-300 transition hover:border-indigo-500 disabled:opacity-40"
        >
          ⬆ 上级
        </button>
        <div className="flex min-w-0 flex-1 items-center gap-2 rounded-md border border-zinc-800 bg-zinc-950 px-3 py-1.5 text-sm text-zinc-300">
          <span className="text-zinc-600">~</span>
          <input
            value={path}
            onChange={(e) => setPath(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && refresh()}
            className="w-full bg-transparent outline-none"
          />
        </div>
        <button
          onClick={refresh}
          disabled={!selectedHostId}
          className="rounded-md bg-indigo-500 px-3 py-1.5 text-sm font-medium text-white transition hover:bg-indigo-400 disabled:opacity-40"
        >
          刷新
        </button>
        <button
          onClick={() => fileInputRef.current?.click()}
          disabled={!selectedHostId || uploading}
          className="rounded-md border border-zinc-600 px-3 py-1.5 text-sm text-zinc-300 transition hover:border-indigo-500 disabled:opacity-40"
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

      {error && <p className="px-3 py-2 text-sm text-red-400">{error}</p>}

      <div className="min-h-0 flex-1 overflow-auto">
        <table className="w-full text-sm">
          <thead className="sticky top-0 bg-zinc-900 text-left text-xs text-zinc-500">
            <tr>
              <th className="px-4 py-2">名称</th>
              <th className="px-4 py-2">大小</th>
              <th className="px-4 py-2">权限</th>
              <th className="px-4 py-2">修改时间</th>
              <th className="px-4 py-2 text-right">操作</th>
            </tr>
          </thead>
          <tbody>
            {loading && (
              <tr>
                <td className="px-4 py-3 text-zinc-600">加载中...</td>
              </tr>
            )}
            {!loading && entries.length === 0 && (
              <tr>
                <td className="px-4 py-3 text-zinc-600">空目录</td>
              </tr>
            )}
            {entries.map((entry) => (
              <tr
                key={entry.path}
                className="border-t border-zinc-800/60 hover:bg-zinc-800/40"
              >
                <td
                  className="cursor-pointer px-4 py-2"
                  onClick={() => entry.is_dir && goTo(entry.path)}
                >
                  <span className="mr-2">
                    {entry.is_dir ? "📁" : "📄"}
                  </span>
                  {entry.name}
                </td>
                <td className="px-4 py-2 font-mono text-xs text-zinc-400">
                  {entry.is_dir ? "-" : formatSize(entry.size)}
                </td>
                <td className="px-4 py-2 font-mono text-xs text-zinc-500">
                  {permString(entry.permissions)}
                </td>
                <td className="px-4 py-2 text-xs text-zinc-500">
                  {entry.modified}
                </td>
                <td className="px-4 py-2 text-right">
                  <div className="flex justify-end gap-1.5 text-xs">
                    {!entry.is_dir && (
                      <>
                        <button
                          onClick={() => openEditor(entry)}
                          className="rounded border border-zinc-700 px-2 py-1 text-zinc-300 hover:border-indigo-500"
                        >
                          编辑
                        </button>
                        <button
                          onClick={() => handleDownload(entry)}
                          className="rounded border border-zinc-700 px-2 py-1 text-zinc-300 hover:border-indigo-500"
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
                      className="rounded border border-zinc-700 px-2 py-1 text-zinc-300 hover:border-indigo-500"
                    >
                      重命名
                    </button>
                    <button
                      onClick={() => handleDelete(entry)}
                      className="rounded border border-red-500/30 px-2 py-1 text-red-400 hover:bg-red-500/10"
                    >
                      删除
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
              <span className="font-mono text-sm text-zinc-300">
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
          <div className="w-96 rounded-lg border border-zinc-700 bg-zinc-950 p-4">
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
