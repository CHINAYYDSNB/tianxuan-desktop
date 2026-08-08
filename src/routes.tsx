import { createBrowserRouter } from "react-router-dom";

import Layout from "./components/layout/Layout";
import Dashboard from "./components/dashboard/Dashboard";
import HostList from "./components/host/HostList";
import HostForm from "./components/host/HostForm";
import BatchCommand from "./components/batch/BatchCommand";
import Settings from "./components/settings/Settings";
import HostEditPage from "./components/host/HostEditPage";

export const router = createBrowserRouter([
  {
    path: "/",
    element: <Layout />,
    children: [
      { index: true, element: <Dashboard /> },
      { path: "hosts", element: <HostList /> },
      { path: "hosts/new", element: <HostForm /> },
      { path: "hosts/:id/edit", element: <HostEditPage /> },
      { path: "batch", element: <BatchCommand /> },
      { path: "settings", element: <Settings /> },
    ],
  },
]);
