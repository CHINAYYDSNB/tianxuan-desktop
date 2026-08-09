import { createBrowserRouter } from "react-router-dom";

import Layout from "./components/layout/Layout";
import Dashboard from "./components/dashboard/Dashboard";
import HostList from "./components/host/HostList";
import HostForm from "./components/host/HostForm";
import BatchCommand from "./components/batch/BatchCommand";
import Settings from "./components/settings/Settings";
import HostEditPage from "./components/host/HostEditPage";
import HostWorkspace from "./components/host/HostWorkspace";
import PanelList from "./components/panel/PanelList";
import PanelForm from "./components/panel/PanelForm";
import PanelEditPage from "./components/panel/PanelEditPage";

export const router = createBrowserRouter([
  {
    path: "/",
    element: <Layout />,
    children: [
      { index: true, element: <Dashboard /> },
      { path: "hosts", element: <HostList /> },
      { path: "hosts/new", element: <HostForm /> },
      { path: "hosts/:id/edit", element: <HostEditPage /> },
      { path: "hosts/:id/workspace", element: <HostWorkspace /> },
      { path: "panels", element: <PanelList /> },
      { path: "panels/new", element: <PanelForm /> },
      { path: "panels/:id/edit", element: <PanelEditPage /> },
      { path: "batch", element: <BatchCommand /> },
      { path: "settings", element: <Settings /> },
    ],
  },
]);
