import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";
import { usePanelBrowserStore } from "./panelBrowserStore";

const mockedInvoke = vi.mocked(invoke);

describe("panelBrowserStore", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    usePanelBrowserStore.setState({
      active: false,
      tabs: [],
      activeTab: null,
    });
  });

  it("enter opens a panel tab and activates browser mode", async () => {
    mockedInvoke.mockResolvedValue("panel-tab-p1");
    await usePanelBrowserStore.getState().enter("p1", "BT Prod");

    expect(mockedInvoke).toHaveBeenCalledWith("open_panel_tab", { id: "p1" });
    const s = usePanelBrowserStore.getState();
    expect(s.active).toBe(true);
    expect(s.activeTab).toBe("panel-tab-p1");
    expect(s.tabs).toEqual([{ label: "panel-tab-p1", name: "BT Prod" }]);
  });

  it("switchTab calls the backend and updates activeTab", async () => {
    usePanelBrowserStore.setState({
      active: true,
      tabs: [
        { label: "panel-tab-p1", name: "A" },
        { label: "panel-tab-p2", name: "B" },
      ],
      activeTab: "panel-tab-p1",
    });
    mockedInvoke.mockResolvedValue(undefined);
    await usePanelBrowserStore.getState().switchTab("panel-tab-p2");
    expect(mockedInvoke).toHaveBeenCalledWith("switch_panel_tab", { label: "panel-tab-p2" });
    expect(usePanelBrowserStore.getState().activeTab).toBe("panel-tab-p2");
  });

  it("closeTab removes a tab and falls back activeTab", async () => {
    usePanelBrowserStore.setState({
      active: true,
      tabs: [
        { label: "panel-tab-p1", name: "A" },
        { label: "panel-tab-p2", name: "B" },
      ],
      activeTab: "panel-tab-p2",
    });
    mockedInvoke.mockResolvedValue(undefined);
    await usePanelBrowserStore.getState().closeTab("panel-tab-p2");
    expect(mockedInvoke).toHaveBeenCalledWith("close_panel_tab", { label: "panel-tab-p2" });
    const s = usePanelBrowserStore.getState();
    expect(s.tabs).toEqual([{ label: "panel-tab-p1", name: "A" }]);
    expect(s.activeTab).toBe("panel-tab-p1");
  });

  it("exit hides all tabs and leaves browser mode", async () => {
    mockedInvoke.mockResolvedValue(undefined);
    await usePanelBrowserStore.getState().exit();
    expect(mockedInvoke).toHaveBeenCalledWith("hide_panel_tabs");
    expect(usePanelBrowserStore.getState().active).toBe(false);
  });
});
