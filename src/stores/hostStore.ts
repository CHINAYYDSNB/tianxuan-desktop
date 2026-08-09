import { create } from "zustand";
import {
  addHost as apiAddHost,
  deleteHost as apiDeleteHost,
  listHosts as apiListHosts,
  type Host,
} from "../lib/tauri";

interface HostStore {
  hosts: Host[];
  loading: boolean;
  error: string | null;
  load: () => Promise<void>;
  add: (host: Omit<Host, "id" | "created_at" | "updated_at" | "auth_ref">, password?: string) => Promise<void>;
  remove: (id: string) => Promise<void>;
}

export const useHostStore = create<HostStore>((set) => ({
  hosts: [],
  loading: false,
  error: null,

  load: async () => {
    set({ loading: true, error: null });
    try {
      const hosts = await apiListHosts();
      set({ hosts, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  add: async (host, password) => {
    await apiAddHost(host, password);
    await useHostStore.getState().load();
  },

  remove: async (id) => {
    await apiDeleteHost(id);
    await useHostStore.getState().load();
  },
}));
