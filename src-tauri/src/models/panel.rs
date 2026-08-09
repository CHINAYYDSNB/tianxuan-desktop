use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PanelType {
    Bt,
    OnePanel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Panel {
    pub id: String,
    pub name: String,
    pub url: String,
    pub panel_type: PanelType,
    pub session_ref: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewPanel {
    pub name: String,
    pub url: String,
    #[serde(default = "default_panel_type")]
    pub panel_type: PanelType,
}

fn default_panel_type() -> PanelType {
    PanelType::Bt
}

impl NewPanel {
    pub fn into_panel(self) -> Panel {
        let now = chrono::Utc::now().to_rfc3339();
        Panel {
            id: uuid::Uuid::new_v4().to_string(),
            name: self.name,
            url: self.url,
            panel_type: self.panel_type,
            session_ref: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}
