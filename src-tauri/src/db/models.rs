use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ClipboardItemType {
    Text,
    Image,
    File,
    Url,
    Code,
}

impl ClipboardItemType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Image => "image",
            Self::File => "file",
            Self::Url => "url",
            Self::Code => "code",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "text" => Some(Self::Text),
            "image" => Some(Self::Image),
            "file" => Some(Self::File),
            "url" => Some(Self::Url),
            "code" => Some(Self::Code),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ClipboardFilter {
    All,
    Text,
    Image,
    File,
    Url,
    Pinned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardItem {
    pub id: String,
    #[serde(rename = "type")]
    pub item_type: ClipboardItemType,
    pub content: String,
    pub normalized_content: String,
    pub preview: String,
    pub pinned: bool,
    pub created_at: i64,
}

#[derive(Debug, Clone)]
pub struct NewClipboardItem {
    pub item_type: ClipboardItemType,
    pub content: String,
    pub preview: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum SaveOutcome {
    Inserted(ClipboardItem),
    Duplicate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub launch_on_startup: bool,
    pub enable_monitoring: bool,
    pub max_history_size: u32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            launch_on_startup: true,
            enable_monitoring: true,
            max_history_size: 10_000,
        }
    }
}
