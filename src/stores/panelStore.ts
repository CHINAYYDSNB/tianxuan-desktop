import { create } from "zustand";
import {
  addPanel as apiAddPanel,
  deletePanel as apiDeletePanel,
  listPanels as apiListPanels,
  type Panel,
} from "../lib/tauri";

interface PanelStore {
  panels: Panel[];
  loading: boolean;
  error: string | null;
  load: () => Promise<void>;
  add: (panel: Omit<Panel, "id" | "session_ref" | "created_at" | "updated_at">) => Promise<void>;
  remove: (id: string) => Promise<void>;
}

export const usePanelStore = create<PanelStore>((set) => ({
  panels: [],
  loading: false,
  error: null,

  load: async () => {
    set({ loading: true, error: null });
    try {
      const panels = await apiListPanels();
      set({ panels, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  add: async (panel) => {
    await apiAddPanel(panel);
    await usePanelStore.getState().load();
  },

  remove: async (id) => {
    await apiDeletePanel(id);
    await usePanelStore.getState().load();
  },
}));
