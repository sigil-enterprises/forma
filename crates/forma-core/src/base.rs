use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use forma_schema::content::{ColorConfig, TypographyConfig};

// -- Publishing / style primitives (duplicate of forma-schema for core usage) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PublishingConfig {
    #[serde(default)]
    pub google_drive_folder_id: String,
    #[serde(default)]
    pub filename_prefix: String,
}

impl Default for PublishingConfig {
    fn default() -> Self {
        Self {
            google_drive_folder_id: String::new(),
            filename_prefix: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrandConfig {
    #[serde(default)]
    pub logo: String,
    #[serde(default)]
    pub logo_white: String,
}

impl Default for BrandConfig {
    fn default() -> Self {
        Self {
            logo: String::new(),
            logo_white: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutConfig {
    #[serde(default = "default_page_size")]
    pub page_size: String,
    #[serde(default = "default_slides_aspect_ratio")]
    pub slides_aspect_ratio: String,
}

fn default_page_size() -> String { "a4".into() }
fn default_slides_aspect_ratio() -> String { "169".into() }

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            page_size: default_page_size(),
            slides_aspect_ratio: default_slides_aspect_ratio(),
        }
    }
}

// -- BaseStyle --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaseStyle {
    #[serde(default)]
    pub publishing: PublishingConfig,
}

impl Default for BaseStyle {
    fn default() -> Self {
        Self {
            publishing: Default::default(),
        }
    }
}

impl BaseStyle {
    pub fn from_yaml(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;
        let data: HashMap<String, serde_yaml::Value> = serde_yaml::from_str(&contents)?;
        Ok(Self {
            publishing: data.get("publishing")
                .and_then(|v| serde_yaml::from_value(v.clone()).ok())
                .unwrap_or_default(),
        })
    }
}

// -- FormaStyle --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormaStyle {
    #[serde(default)]
    pub brand: BrandConfig,
    #[serde(default)]
    pub colors: ColorConfig,
    #[serde(default)]
    pub typography: TypographyConfig,
    #[serde(default)]
    pub layout: LayoutConfig,
    #[serde(default)]
    pub publishing: PublishingConfig,
}

impl Default for FormaStyle {
    fn default() -> Self {
        Self {
            brand: Default::default(),
            colors: Default::default(),
            typography: Default::default(),
            layout: Default::default(),
            publishing: Default::default(),
        }
    }
}

impl FormaStyle {
    pub fn from_yaml(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;
        let style: Self = serde_yaml::from_str(&contents)?;
        Ok(style)
    }
}

// -- BaseContent --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaseContent {
    #[serde(default)]
    pub publishing: PublishingConfig,
}

impl Default for BaseContent {
    fn default() -> Self {
        Self {
            publishing: Default::default(),
        }
    }
}

impl BaseContent {
    pub fn from_yaml(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;
        let data: HashMap<String, serde_yaml::Value> = serde_yaml::from_str(&contents)?;
        Ok(Self {
            publishing: data.get("publishing")
                .and_then(|v| serde_yaml::from_value(v.clone()).ok())
                .unwrap_or_default(),
        })
    }
}
