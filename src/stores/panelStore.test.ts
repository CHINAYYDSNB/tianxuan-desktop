import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";
import { usePanelStore } from "./panelStore";

const mockedInvoke = vi.mocked(invoke);

const samplePanels = [
  {
    id: "p1",
    name: "BT Prod",
    url: "https://panel.example.com:8888",
    panel_type: "bt",
    session_ref: null,
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
  },
];

describe("panelStore", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    usePanelStore.setState({ panels: [], loading: false, error: null });
  });

  it("loads panels from invoke", async () => {
    mockedInvoke.mockResolvedValue(samplePanels);
    await usePanelStore.getState().load();
    expect(usePanelStore.getState().panels).toHaveLength(1);
    expect(mockedInvoke).toHaveBeenCalledWith("list_panels");
  });

  it("add calls add_panel then reloads", async () => {
    mockedInvoke.mockResolvedValueOnce(samplePanels[0]);
    mockedInvoke.mockResolvedValueOnce(samplePanels);
    await usePanelStore.getState().add({
      name: "BT Prod",
      url: "https://panel.example.com:8888",
      panel_type: "bt",
    });
    expect(mockedInvoke).toHaveBeenNthCalledWith(
      1,
      "add_panel",
      expect.objectContaining({ panel: expect.any(Object) }),
    );
    expect(mockedInvoke).toHaveBeenNthCalledWith(2, "list_panels");
  });

  it("remove calls delete_panel then reloads", async () => {
    mockedInvoke.mockResolvedValueOnce(undefined);
    mockedInvoke.mockResolvedValueOnce([]);
    await usePanelStore.getState().remove("p1");
    expect(mockedInvoke).toHaveBeenNthCalledWith(1, "delete_panel", { id: "p1" });
    expect(mockedInvoke).toHaveBeenNthCalledWith(2, "list_panels");
  });
});
