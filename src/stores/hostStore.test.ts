import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";
import { useHostStore } from "./hostStore";

const mockedInvoke = vi.mocked(invoke);

const sampleHosts = [
  {
    id: "h1",
    name: "Prod",
    address: "47.100.33.169",
    port: 22,
    username: "root",
    auth_type: "password",
    auth_ref: "ref",
    group_name: "生产",
    tags: ["bt"],
    panel_type: null,
    panel_url: null,
    panel_session_ref: null,
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
  },
  {
    id: "h2",
    name: "Test",
    address: "10.0.0.2",
    port: 22,
    username: "root",
    auth_type: "password",
    auth_ref: "ref2",
    group_name: "测试",
    tags: [],
    panel_type: null,
    panel_url: null,
    panel_session_ref: null,
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
  },
];

describe("hostStore", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useHostStore.setState({ hosts: [], loading: false, error: null });
  });

  it("loads hosts from invoke", async () => {
    mockedInvoke.mockResolvedValue(sampleHosts);
    await useHostStore.getState().load();
    expect(useHostStore.getState().hosts).toHaveLength(2);
    expect(mockedInvoke).toHaveBeenCalledWith("list_hosts");
  });

  it("load sets error on failure", async () => {
    mockedInvoke.mockRejectedValue("db down");
    await useHostStore.getState().load();
    expect(useHostStore.getState().error).toBe("db down");
    expect(useHostStore.getState().loading).toBe(false);
  });

  it("add calls add_host then reloads", async () => {
    mockedInvoke.mockResolvedValueOnce(sampleHosts[0]);
    mockedInvoke.mockResolvedValueOnce(sampleHosts);
    await useHostStore.getState().add({
      name: "Prod",
      address: "47.100.33.169",
      port: 22,
      username: "root",
      auth_type: "password",
      group_name: "生产",
      tags: ["bt"],
      panel_type: null,
      panel_url: null,
    });
    expect(mockedInvoke).toHaveBeenNthCalledWith(
      1,
      "add_host",
      expect.objectContaining({ host: expect.any(Object) }),
    );
    expect(mockedInvoke).toHaveBeenNthCalledWith(2, "list_hosts");
  });

  it("remove calls delete_host then reloads", async () => {
    mockedInvoke.mockResolvedValueOnce(undefined);
    mockedInvoke.mockResolvedValueOnce(sampleHosts.filter((h) => h.id !== "h1"));
    await useHostStore.getState().remove("h1");
    expect(mockedInvoke).toHaveBeenNthCalledWith(1, "delete_host", { id: "h1" });
    expect(mockedInvoke).toHaveBeenNthCalledWith(2, "list_hosts");
  });
});
