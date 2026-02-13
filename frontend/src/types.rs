use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistroTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub category: DistroCategory,
    pub default_packages: Vec<String>,
    pub desktop_environment: String,
    pub base_image: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DistroCategory {
    Ubuntu,
    Debian,
    Arch,
    Fedora,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsoConfig {
    pub id: Uuid,
    pub name: String,
    pub distro: DistroTemplate,
    pub packages: Vec<String>,
    pub custom_scripts: Vec<String>,
    pub desktop_environment: Option<String>,
    pub theme: ThemeConfig,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub wallpaper: Option<String>,
    pub gtk_theme: Option<String>,
    pub icon_theme: Option<String>,
    pub colors: ColorScheme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    pub primary: String,
    pub secondary: String,
    pub background: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildJob {
    pub id: Uuid,
    pub config: IsoConfig,
    pub status: BuildStatus,
    pub progress: u8,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub download_url: Option<String>,
    pub logs: Vec<BuildLog>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BuildStatus {
    Queued,
    Building,
    Packaging,
    Uploading,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildLog {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
    Debug,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIsoRequest {
    pub config: IsoConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub job_id: Uuid,
    pub type_: MessageType,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    StatusUpdate,
    ProgressUpdate,
    LogMessage,
    Completed,
    Error,
}
