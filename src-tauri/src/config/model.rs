use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default, rename = "outputFolder")]
    pub output_folder: String,
    #[serde(
        default = "default_zoom_scale",
        rename = "zoomScale",
        deserialize_with = "deserialize_zoom_scale"
    )]
    pub zoom_scale: f64,
}

fn default_theme() -> String {
    "dark".to_string()
}

fn default_zoom_scale() -> f64 {
    1.0
}

fn deserialize_zoom_scale<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<f64>::deserialize(deserializer)?;

    match opt {
        Some(v) if (0.25..=5.0).contains(&v) => Ok(v),
        _ => Ok(default_zoom_scale()),
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            output_folder: String::new(),
            zoom_scale: default_zoom_scale(),
        }
    }
}
