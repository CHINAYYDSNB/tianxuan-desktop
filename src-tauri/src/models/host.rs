use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AuthType {
    Key,
    Password,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PanelType {
    Bt,
    OnePanel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Host {
    pub id: String,
    pub name: String,
    pub address: String,
    pub port: u16,
    pub username: String,
    pub auth_type: AuthType,
    pub auth_ref: String,
    pub group_name: String,
    pub tags: Vec<String>,
    pub panel_type: Option<PanelType>,
    pub panel_url: Option<String>,
    pub panel_session_ref: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl Host {
    pub fn new(
        name: String,
        address: String,
        port: u16,
        username: String,
        auth_type: AuthType,
        auth_ref: String,
        group_name: String,
        tags: Vec<String>,
        panel_type: Option<PanelType>,
        panel_url: Option<String>,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            address,
            port,
            username,
            auth_type,
            auth_ref,
            group_name,
            tags,
            panel_type,
            panel_url,
            panel_session_ref: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewHost {
    pub name: String,
    pub address: String,
    #[serde(default = "default_port")]
    pub port: u16,
    pub username: String,
    #[serde(default = "default_auth_type")]
    pub auth_type: AuthType,
    pub group_name: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub panel_type: Option<PanelType>,
    #[serde(default)]
    pub panel_url: Option<String>,
}

fn default_port() -> u16 {
    22
}

fn default_auth_type() -> AuthType {
    AuthType::Password
}

impl NewHost {
    pub fn into_host(self) -> Host {
        let now = chrono::Utc::now().to_rfc3339();
        Host {
            id: uuid::Uuid::new_v4().to_string(),
            name: self.name,
            address: self.address,
            port: self.port,
            username: self.username,
            auth_type: self.auth_type,
            auth_ref: String::new(),
            group_name: self.group_name,
            tags: self.tags,
            panel_type: self.panel_type,
            panel_url: self.panel_url,
            panel_session_ref: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}
