import { create } from "zustand";
import {
  closePanelTab as apiCloseTab,
  hidePanelTabs as apiHideTabs,
  listPanelTabs as apiListTabs,
  openPanelTab as apiOpenTab,
  switchPanelTab as apiSwitchTab,
} from "../lib/tauri";

interface PanelBrowserState {
  active: boolean;
  tabs: { label: string; name: string }[];
  activeTab: string | null;
  enter: (panelId: string, panelName: string) => Promise<void>;
  switchTab: (label: string) => Promise<void>;
  closeTab: (label: string) => Promise<void>;
  exit: () => Promise<void>;
  refreshTabs: () => Promise<void>;
}

export const usePanelBrowserStore = create<PanelBrowserState>((set) => ({
  active: false,
  tabs: [],
  activeTab: null,

  refreshTabs: async () => {
    try {
      const labels = await apiListTabs();
      set({ tabs: labels.map((label) => ({ label, name: label })) });
    } catch {
      set({ tabs: [] });
    }
  },

  enter: async (panelId, panelName) => {
    const label = await apiOpenTab(panelId);
    set((s) => ({
      active: true,
      activeTab: label,
      tabs: s.tabs.some((t) => t.label === label)
        ? s.tabs
        : [...s.tabs, { label, name: panelName }],
    }));
  },

  switchTab: async (label) => {
    await apiSwitchTab(label);
    set({ activeTab: label });
  },

  closeTab: async (label) => {
    await apiCloseTab(label);
    set((s) => {
      const tabs = s.tabs.filter((t) => t.label !== label);
      const activeTab =
        s.activeTab === label ? (tabs.length ? tabs[tabs.length - 1].label : null) : s.activeTab;
      return { tabs, activeTab };
    });
  },

  exit: async () => {
    await apiHideTabs();
    set({ active: false });
  },
}));
