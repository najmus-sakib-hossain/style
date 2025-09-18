use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct PathsConfig {
    pub html_dir: String,
    pub index_file: String,
    pub css_file: String,
    #[serde(default)]
    pub style_dir: Option<String>,
    #[serde(default)]
    pub cache_dir: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WatchConfig {
    pub debounce_ms: Option<u64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub paths: PathsConfig,
    pub watch: Option<WatchConfig>,
    #[serde(default)]
    pub format: Option<FormatConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FormatConfig {
    #[serde(default = "FormatConfig::default_delay")]
    pub delay_ms: u64,
    #[serde(default = "FormatConfig::default_interval")]
    pub interval_ms: u64,
    #[serde(default)]
    pub force_write: bool,
    #[serde(default = "FormatConfig::default_debounce")]
    pub debounce_ms: u64,
}

impl FormatConfig {
    fn default_delay() -> u64 {
        10_000
    }
    fn default_interval() -> u64 {
        10_000
    }
    fn default_debounce() -> u64 {
        1_000
    }
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(".dx/config.toml")?;
        let cfg: Config = toml::from_str(&content)?;
        Ok(cfg)
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            paths: PathsConfig {
                html_dir: "playgrounds/html".into(),
                index_file: "playgrounds/html/index.html".into(),
                css_file: "playgrounds/html/style.css".into(),
                style_dir: Some(".dx/style".into()),
                cache_dir: Some(".dx/cache".into()),
            },
            watch: Some(WatchConfig {
                debounce_ms: Some(250),
            }),
            format: Some(FormatConfig {
                delay_ms: FormatConfig::default_delay(),
                interval_ms: FormatConfig::default_interval(),
                force_write: false,
                debounce_ms: FormatConfig::default_debounce(),
            }),
        }
    }
}

impl Config {
    pub fn resolved_style_dir(&self) -> &str {
        self.paths.style_dir.as_deref().unwrap_or(".dx/style")
    }
    pub fn resolved_cache_dir(&self) -> &str {
        self.paths.cache_dir.as_deref().unwrap_or(".dx/cache")
    }
    pub fn format_delay_ms(&self) -> u64 {
        self.format
            .as_ref()
            .map(|f| f.delay_ms)
            .unwrap_or(FormatConfig::default_delay())
    }
    pub fn format_interval_ms(&self) -> u64 {
        self.format
            .as_ref()
            .map(|f| f.interval_ms)
            .unwrap_or(FormatConfig::default_interval())
    }
    pub fn format_force_write(&self) -> bool {
        self.format.as_ref().map(|f| f.force_write).unwrap_or(false)
    }
    pub fn format_debounce_ms(&self) -> u64 {
        self.format
            .as_ref()
            .map(|f| f.debounce_ms)
            .unwrap_or(FormatConfig::default_debounce())
    }
}
