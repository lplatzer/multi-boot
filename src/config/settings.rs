use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub default_label: String,
    pub timeout: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            default_label: "MULTIBOOT".to_string(),
            timeout: 30,
        }
    }
}
