use std::collections::HashMap;
use std::sync::Mutex;

use tauri::{
    LogicalPosition, LogicalSize, Webview, WebviewUrl, Window, webview::WebviewBuilder,
};

use crate::models::Panel;

/// Height in logical px reserved for the React tab bar at the top.
pub const TOOLBAR_HEIGHT: f64 = 40.0;

pub struct PanelTabManager {
    tabs: Mutex<HashMap<String, Webview>>,
    active: Mutex<Option<String>>,
}

impl PanelTabManager {
    pub fn new() -> Self {
        Self {
            tabs: Mutex::new(HashMap::new()),
            active: Mutex::new(None),
        }
    }

    fn tab_label(panel_id: &str) -> String {
        format!("panel-tab-{panel_id}")
    }

    fn layout_for(window: &Window) -> (LogicalPosition<f64>, LogicalSize<f64>) {
        let size = window.inner_size().unwrap_or_default();
        let scale = window.scale_factor().unwrap_or(1.0);
        let width = size.width as f64 / scale;
        let height = size.height as f64 / scale - TOOLBAR_HEIGHT;
        (
            LogicalPosition::new(0.0, TOOLBAR_HEIGHT),
            LogicalSize::new(width, height.max(0.0)),
        )
    }

    /// Open (or bring to front) the panel webview for the given panel.
    /// `window` is the parent Window (cloned by the caller).
    pub fn open(
        &self,
        window: &Window,
        panel: &Panel,
    ) -> Result<String, String> {
        let label = Self::tab_label(&panel.id);

        {
            let tabs = self.tabs.lock().unwrap();
            if let Some(wv) = tabs.get(&label) {
                let _ = wv.show();
                let _ = wv.set_focus();
                *self.active.lock().unwrap() = Some(label.clone());
                return Ok(label);
            }
        }

        let url = panel
            .url
            .parse::<tauri::Url>()
            .map_err(|e| format!("invalid panel URL: {e}"))?;

        let builder = WebviewBuilder::new(label.clone(), WebviewUrl::External(url));

        let (pos, size) = Self::layout_for(window);
        let webview = window
            .add_child(builder, pos, size)
            .map_err(|e| format!("add_child failed: {e}"))?;

        {
            let mut tabs = self.tabs.lock().unwrap();
            if let Some(cur) = self.active.lock().unwrap().clone() {
                if let Some(wv) = tabs.get(&cur) {
                    let _ = wv.hide();
                }
            }
            let _ = webview.show();
            tabs.insert(label.clone(), webview);
            *self.active.lock().unwrap() = Some(label.clone());
        }

        Ok(label)
    }

    /// Switch active tab: show target, hide others.
    pub fn switch(&self, label: &str) -> Result<(), String> {
        let tabs = self.tabs.lock().unwrap();
        for (l, wv) in tabs.iter() {
            if l == label {
                let _ = wv.show();
                let _ = wv.set_focus();
            } else {
                let _ = wv.hide();
            }
        }
        if tabs.contains_key(label) {
            *self.active.lock().unwrap() = Some(label.to_string());
        }
        Ok(())
    }

    /// Hide all panel tabs (return to app UI). Instances are kept alive.
    pub fn hide_all(&self) {
        let tabs = self.tabs.lock().unwrap();
        for wv in tabs.values() {
            let _ = wv.hide();
        }
    }

    /// Close & destroy a specific tab.
    pub fn close(&self, label: &str) -> Result<(), String> {
        let mut tabs = self.tabs.lock().unwrap();
        if let Some(wv) = tabs.remove(label) {
            let _ = wv.close();
            if self.active.lock().unwrap().as_deref() == Some(label) {
                *self.active.lock().unwrap() = None;
            }
        }
        Ok(())
    }

    /// Resize all tabs to match the given window size.
    pub fn resize_all(&self, window: &Window) {
        let (pos, size) = Self::layout_for(window);
        let tabs = self.tabs.lock().unwrap();
        for wv in tabs.values() {
            let bounds = tauri::Rect {
                position: pos.into(),
                size: size.into(),
            };
            let _ = wv.set_bounds(bounds);
        }
    }

    /// Return the list of currently open tab labels.
    pub fn open_labels(&self) -> Vec<String> {
        self.tabs.lock().unwrap().keys().cloned().collect()
    }

    pub fn active_label(&self) -> Option<String> {
        self.active.lock().unwrap().clone()
    }
}
