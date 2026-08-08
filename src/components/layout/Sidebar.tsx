import { NavLink } from "react-router-dom";

const navItems = [
  { to: "/", label: "总览", icon: "▦" },
  { to: "/hosts", label: "主机", icon: "🖥" },
  { to: "/batch", label: "批量", icon: "⚡" },
  { to: "/settings", label: "设置", icon: "⚙" },
];

export default function Sidebar() {
  return (
    <aside className="flex w-52 shrink-0 flex-col border-r border-zinc-800 bg-zinc-950">
      <div className="flex h-14 items-center gap-2 border-b border-zinc-800 px-4">
        <span className="text-lg">🛰</span>
        <span className="font-semibold tracking-tight text-indigo-400">
          Tianxuan
        </span>
      </div>
      <nav className="flex flex-1 flex-col gap-1 p-2">
        {navItems.map((item) => (
          <NavLink
            key={item.to}
            to={item.to}
            className={({ isActive }) =>
              `flex items-center gap-3 rounded-md px-3 py-2 text-sm transition ${
                isActive
                  ? "bg-indigo-500/15 text-indigo-300"
                  : "text-zinc-400 hover:bg-zinc-800/60 hover:text-zinc-200"
              }`
            }
          >
            <span>{item.icon}</span>
            <span>{item.label}</span>
          </NavLink>
        ))}
      </nav>
      <div className="border-t border-zinc-800 p-3 text-xs text-zinc-600">
        v0.1.0
      </div>
    </aside>
  );
}
