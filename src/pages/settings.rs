use keycast::discovery::Discovery;
use serde::{Deserialize, Serialize};

use base64::{engine::general_purpose, Engine as _};
use rand::RngCore;
use uuid::Uuid;
use verdant::livekit::TokenResponse;

fn random_base64_string() -> String {
    let mut bytes = [0u8; 16]; // 16 random bytes
    rand::thread_rng().fill_bytes(&mut bytes);
    general_purpose::STANDARD.encode(&bytes)
}

fn main() {
    let s = random_base64_string();
    println!("Random base64 string: {}", s);
}

/// a trait to describe general settings needed for the app to operate.
pub trait Settings {
    fn auto_subscribe(&self) -> bool {
        true
    }
    fn auto_publish(&self) -> bool {
        false
    }
    /// should end to end encryption be enabled
    fn enable_e2ee(&self) -> bool {
        true
    }
    /// should auto discovery of servers be enabled
    fn use_discovery(&self) -> bool {
        true
    }
}

/// per server manual configuration.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ServerSettings {
    pub settings: GeneralSettings,
    pub name: String,
    pub url: String,
    pub token: String,
}

impl ServerSettings {
    pub fn set_token(&mut self, token: &str) {
        self.token = token.to_string();
    }

    pub fn set_url(&mut self, url: &str) {
        self.url = url.to_string();
    }
    pub fn from_discovery(discovery: &Discovery) -> Self {
        let mut settings = GeneralSettings::default();
        settings.enable_e2ee = true;
        let name = discovery.host.clone();
        let url = format!("{}:{}", discovery.addrs.get(0).unwrap(), discovery.port);
        Self {
            settings,
            name,
            url,
            // set to an empty string initially
            token: "".to_string(),
        }
    }
    pub fn from_response(settings: &GeneralSettings, ident: &str, response: &TokenResponse) -> Self {
        Self {
            settings: settings.clone(),
            name: ident.to_string(),
            url: response.url.to_string(),
            token: response.token.to_string(),
        }
    }
}

impl Settings for ServerSettings {
    fn auto_publish(&self) -> bool {
        self.settings.auto_publish()
    }

    fn auto_subscribe(&self) -> bool {
        self.settings.auto_subscribe()
    }

    fn enable_e2ee(&self) -> bool {
        self.settings.enable_e2ee()
    }

    fn use_discovery(&self) -> bool {
        self.settings.use_discovery()
    }
}

/// per room manual configuration settings.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct RoomSettings {
    pub id: Uuid,
    pub server: ServerSettings,
    pub name: String,
    pub key: String,
}

impl RoomSettings {
    pub fn id(&self) -> Uuid {
        self.id
    }
    pub fn set_token(&mut self, token: &str) {
        self.server.set_token(token);
    }
    pub fn set_url(&mut self, url: &str) {
        self.server.set_url(url)
    }

    pub fn from_response(settings: &GeneralSettings, ident: &str, response: &TokenResponse) -> Self {
        let server = ServerSettings::from_response(settings, ident, response);
        Self {
            id: response.room_id,
            server,
            name: response.room.clone(),
            key: random_base64_string(),
        }
    }
}

impl RoomSettings {
    pub fn token(&self) -> &str {
        &self.server.token
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn url(&self) -> &str {
        &self.server.url
    }
}

impl Settings for RoomSettings {
    fn auto_publish(&self) -> bool {
        self.server.auto_publish()
    }

    fn auto_subscribe(&self) -> bool {
        self.server.auto_subscribe()
    }

    fn enable_e2ee(&self) -> bool {
        self.server.enable_e2ee()
    }

    fn use_discovery(&self) -> bool {
        self.server.use_discovery()
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct GeneralSettings {
    pub auto_subscribe: bool,
    pub auto_publish: bool,
    pub enable_e2ee: bool,
    pub use_discovery: bool,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            auto_subscribe: true,
            auto_publish: false,
            enable_e2ee: false,
            use_discovery: true,
        }
    }
}

impl Settings for GeneralSettings {
    fn auto_publish(&self) -> bool {
        self.auto_publish
    }

    fn auto_subscribe(&self) -> bool {
        self.auto_subscribe
    }

    fn enable_e2ee(&self) -> bool {
        self.enable_e2ee
    }

    fn use_discovery(&self) -> bool {
        self.use_discovery
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SettingsPage {
    state: GeneralSettings,
    servers: Vec<ServerSettings>,
}

impl SettingsPage {
    pub fn new(runtime: &tokio::runtime::Runtime, cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            state: GeneralSettings::default(),
            servers: Vec::new(),
        }
    }
}
