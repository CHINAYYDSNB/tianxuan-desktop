import { useEffect, useRef } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

export const FOREGROUND_INTERVAL_MS = 1000;
export const BACKGROUND_INTERVAL_MS = 10000;

/**
 * Poll a callback at a foreground-friendly interval when the app window is
 * focused & visible, and a relaxed interval when the window is in the
 * background. Force a refresh immediately when returning to the foreground.
 *
 * `cb` should be idempotent and guarded against out-of-order results.
 */
export function useVisibilityPolling(
  cb: () => void,
  foregroundMs: number = FOREGROUND_INTERVAL_MS,
  backgroundMs: number = BACKGROUND_INTERVAL_MS,
) {
  const cbRef = useRef(cb);
  cbRef.current = cb;
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  useEffect(() => {
    let cancelled = false;
    let focused = true;

    function clearTimer() {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
        intervalRef.current = null;
      }
    }

    function schedule(focusedNow: boolean) {
      if (cancelled) return;
      clearTimer();
      const ms = focusedNow ? foregroundMs : backgroundMs;
      intervalRef.current = setInterval(() => {
        cbRef.current();
      }, ms);
    }

    function updateFocused(newFocused: boolean) {
      const wasFocused = focused;
      focused = newFocused;
      schedule(focused);
      // returning to the foreground: force an immediate refresh
      if (!wasFocused && focused) {
        cbRef.current();
      }
    }

    cbRef.current();
    schedule(true);

    let unlistenFocus: (() => void) | undefined;
    let visibilityHandler: (() => void) | undefined;

    getCurrentWindow()
      .onFocusChanged(({ payload }) => {
        updateFocused(payload && !document.hidden);
      })
      .then((un) => {
        if (!cancelled) unlistenFocus = un;
        else un();
      })
      .catch(() => {});

    visibilityHandler = () => {
      updateFocused(focused && !document.hidden);
    };
    document.addEventListener("visibilitychange", visibilityHandler);

    return () => {
      cancelled = true;
      clearTimer();
      if (visibilityHandler) {
        document.removeEventListener("visibilitychange", visibilityHandler);
      }
      unlistenFocus?.();
    };
  }, [foregroundMs, backgroundMs]);
}
