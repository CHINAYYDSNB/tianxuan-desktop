import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act } from "@testing-library/react";

const focusHandlerHolder: { current: ((p: { payload: boolean }) => void) | null } = {
  current: null,
};

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    onFocusChanged: (handler: (p: { payload: boolean }) => void) => {
      focusHandlerHolder.current = handler;
      return Promise.resolve(() => {});
    },
  }),
}));

import { useVisibilityPolling } from "./useVisibilityPolling";

let hiddenState = false;

describe("useVisibilityPolling", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    focusHandlerHolder.current = null;
    Object.defineProperty(document, "hidden", {
      configurable: true,
      get: () => hiddenState,
    });
    hiddenState = false;
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("calls the callback immediately on mount", () => {
    const cb = vi.fn();
    renderHook(() => useVisibilityPolling(cb, 1000, 10000));
    expect(cb).toHaveBeenCalledTimes(1);
  });

  it("polls at the foreground interval when focused & visible", async () => {
    const cb = vi.fn();
    renderHook(() => useVisibilityPolling(cb, 1000, 10000));
    expect(cb).toHaveBeenCalledTimes(1);

    await act(async () => {
      vi.advanceTimersByTime(1000);
    });
    expect(cb).toHaveBeenCalledTimes(2);

    await act(async () => {
      vi.advanceTimersByTime(3000);
    });
    // 3 more ticks at 1s
    expect(cb).toHaveBeenCalledTimes(5);
  });

  it("switches to the background interval when focus is lost", async () => {
    const cb = vi.fn();
    renderHook(() => useVisibilityPolling(cb, 1000, 10000));
    expect(cb).toHaveBeenCalledTimes(1);

    await act(async () => {
      focusHandlerHolder.current?.({ payload: false });
    });
    // losing focus does not trigger immediate cb
    expect(cb).toHaveBeenCalledTimes(1);

    await act(async () => {
      vi.advanceTimersByTime(10000);
    });
    // one background tick
    expect(cb).toHaveBeenCalledTimes(2);

    await act(async () => {
      vi.advanceTimersByTime(2000);
    });
    // still 2 (background is 10s)
    expect(cb).toHaveBeenCalledTimes(2);
  });

  it("forces an immediate refresh when returning to the foreground", async () => {
    const cb = vi.fn();
    renderHook(() => useVisibilityPolling(cb, 1000, 10000));
    expect(cb).toHaveBeenCalledTimes(1);

    await act(async () => {
      focusHandlerHolder.current?.({ payload: false });
    });
    await act(async () => {
      vi.advanceTimersByTime(10000);
    });
    expect(cb).toHaveBeenCalledTimes(2);

    await act(async () => {
      focusHandlerHolder.current?.({ payload: true });
    });
    // back to foreground -> immediate forced refresh
    expect(cb).toHaveBeenCalledTimes(3);
  });

  it("treats a hidden document as background even when focused", async () => {
    const cb = vi.fn();
    renderHook(() => useVisibilityPolling(cb, 1000, 10000));

    await act(async () => {
      hiddenState = true;
      document.dispatchEvent(new Event("visibilitychange"));
    });

    await act(async () => {
      vi.advanceTimersByTime(10000);
    });
    // background tick only
    expect(cb).toHaveBeenCalledTimes(2);
  });
});
