import { useState } from "react";
import type { FormEvent } from "react";
import { useNavigate } from "react-router-dom";

import { useHostStore } from "../../stores/hostStore";
import { testConnection, type Host } from "../../lib/tauri";

interface HostFormProps {
  editing?: Host;
}

export default function HostForm({ editing }: HostFormProps) {
  const navigate = useNavigate();
  const { add, remove } = useHostStore();

  const [name, setName] = useState(editing?.name ?? "");
  const [address, setAddress] = useState(editing?.address ?? "");
  const [port, setPort] = useState(editing?.port ?? 22);
  const [username, setUsername] = useState(editing?.username ?? "root");
  const [authType, setAuthType] = useState<"password" | "key">(
    editing?.auth_type ?? "password",
  );
  const [password, setPassword] = useState("");
  const [group, setGroup] = useState(editing?.group_name ?? "默认");
  const [tags, setTags] = useState(editing?.tags.join(",") ?? "");
  const [panelUrl, setPanelUrl] = useState(editing?.panel_url ?? "");
  const [error, setError] = useState<string | null>(null);
  const [testing, setTesting] = useState(false);
  const [testOk, setTestOk] = useState<boolean | null>(null);

  async function handleTest() {
    if (!editing) return;
    setTesting(true);
    setTestOk(null);
    setError(null);
    try {
      await testConnection(editing.id);
      setTestOk(true);
    } catch (e) {
      setError(String(e));
      setTestOk(false);
    } finally {
      setTesting(false);
    }
  }

  async function handleSubmit(e: FormEvent) {
    e.preventDefault();
    setError(null);
    const tagList = tags
      .split(",")
      .map((t) => t.trim())
      .filter(Boolean);
    const payload = {
      name,
      address,
      port: Number(port) || 22,
      username,
      auth_type: authType,
      group_name: group,
      tags: tagList,
      panel_type: panelUrl ? (editing?.panel_type ?? "bt") : null,
      panel_url: panelUrl || null,
    } as const;
    try {
      await add(payload as never, password || undefined);
      navigate("/hosts");
    } catch (err) {
      setError(String(err));
    }
  }

  return (
    <div className="max-w-xl p-6">
      <h1 className="mb-6 text-xl font-semibold">
        {editing ? "编辑主机" : "添加主机"}
      </h1>

      <form onSubmit={handleSubmit} className="flex flex-col gap-4">
        <label className="flex flex-col gap-1 text-sm">
          <span className="text-zinc-400">显示名称</span>
          <input
            className="rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 outline-none focus:border-indigo-500"
            value={name}
            onChange={(e) => setName(e.target.value)}
            required
          />
        </label>

        <div className="flex gap-3">
          <label className="flex flex-1 flex-col gap-1 text-sm">
            <span className="text-zinc-400">IP / 域名</span>
            <input
              className="rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 outline-none focus:border-indigo-500"
              value={address}
              onChange={(e) => setAddress(e.target.value)}
              required
            />
          </label>
          <label className="flex w-28 flex-col gap-1 text-sm">
            <span className="text-zinc-400">端口</span>
            <input
              type="number"
              className="rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 outline-none focus:border-indigo-500"
              value={port}
              onChange={(e) => setPort(Number(e.target.value))}
            />
          </label>
        </div>

        <div className="flex gap-3">
          <label className="flex flex-1 flex-col gap-1 text-sm">
            <span className="text-zinc-400">用户名</span>
            <input
              className="rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 outline-none focus:border-indigo-500"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              required
            />
          </label>
          <label className="flex w-40 flex-col gap-1 text-sm">
            <span className="text-zinc-400">认证方式</span>
            <select
              className="rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 outline-none focus:border-indigo-500"
              value={authType}
              onChange={(e) => setAuthType(e.target.value as "password" | "key")}
            >
              <option value="password">密码</option>
              <option value="key">私钥</option>
            </select>
          </label>
        </div>

        {authType === "password" && (
          <label className="flex flex-col gap-1 text-sm">
            <span className="text-zinc-400">
              {editing ? "新密码（留空不修改）" : "密码"}
            </span>
            <input
              type="password"
              className="rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 outline-none focus:border-indigo-500"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              required={!editing}
            />
          </label>
        )}

        <div className="flex gap-3">
          <label className="flex flex-1 flex-col gap-1 text-sm">
            <span className="text-zinc-400">分组</span>
            <input
              className="rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 outline-none focus:border-indigo-500"
              value={group}
              onChange={(e) => setGroup(e.target.value)}
            />
          </label>
          <label className="flex flex-1 flex-col gap-1 text-sm">
            <span className="text-zinc-400">标签（逗号分隔）</span>
            <input
              className="rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 outline-none focus:border-indigo-500"
              value={tags}
              onChange={(e) => setTags(e.target.value)}
            />
          </label>
        </div>

        <label className="flex flex-col gap-1 text-sm">
          <span className="text-zinc-400">面板地址（可选，支持宝塔 / 1Panel）</span>
          <input
            className="rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 outline-none focus:border-indigo-500"
            value={panelUrl}
            onChange={(e) => setPanelUrl(e.target.value)}
            placeholder="https://panel.example.com:8888"
          />
        </label>

        {error && <p className="text-sm text-red-400">{error}</p>}
        {testOk && (
          <p className="text-sm text-green-400">连接测试成功</p>
        )}

        <div className="flex gap-2">
          <button
            type="submit"
            className="rounded-md bg-indigo-500 px-4 py-2 text-sm font-medium text-white transition hover:bg-indigo-400"
          >
            {editing ? "保存修改" : "添加主机"}
          </button>
          {editing && (
            <button
              type="button"
              onClick={handleTest}
              disabled={testing}
              className="rounded-md border border-zinc-600 px-4 py-2 text-sm text-zinc-300 transition hover:border-indigo-500"
            >
              {testing ? "测试中..." : "测试连接"}
            </button>
          )}
          {editing && (
            <button
              type="button"
              onClick={async () => {
                if (confirm(`删除主机 ${editing.name}？`)) {
                  await remove(editing.id);
                  navigate("/hosts");
                }
              }}
              className="rounded-md border border-red-500/40 px-4 py-2 text-sm text-red-400 transition hover:bg-red-500/10"
            >
              删除
            </button>
          )}
        </div>
      </form>
    </div>
  );
}
