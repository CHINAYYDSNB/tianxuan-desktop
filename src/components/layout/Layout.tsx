import { Outlet } from "react-router-dom";

import Sidebar from "./Sidebar";
import PanelBrowser from "../panel/PanelBrowser";

export default function Layout() {
  return (
    <div className="flex h-screen bg-zinc-900 text-zinc-100">
      <Sidebar />
      <main className="flex-1 overflow-auto">
        <Outlet />
      </main>
      <PanelBrowser />
    </div>
  );
}
