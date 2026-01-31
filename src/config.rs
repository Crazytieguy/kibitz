use ratatui::style::Color;
use serde::Deserialize;
use std::path::Path;

/// Layout mode for the file tree
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LayoutMode {
    #[default]
    Vertical,
    Horizontal,
}

/// Layout configuration
#[derive(Debug, Clone)]
pub struct LayoutConfig {
    pub mode: LayoutMode,
    pub max_rows: u16,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            mode: LayoutMode::Vertical,
            max_rows: 5,
        }
    }
}

/// Top-level configuration
#[derive(Debug, Clone, Default)]
pub struct Config {
    pub delta: DeltaConfig,
    pub colors: ColorConfig,
    pub layout: LayoutConfig,
}

/// Delta pass-through configuration
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct DeltaConfig {
    /// Additional args appended to delta command
    /// Example: "--side-by-side --line-numbers"
    pub args: Option<String>,
}

/// Semantic color configuration
#[derive(Debug, Clone)]
pub struct ColorConfig {
    pub text: Color,
    pub text_muted: Color,
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,
}

impl Default for ColorConfig {
    fn default() -> Self {
        Self {
            text: Color::Reset,          // Terminal default
            text_muted: Color::DarkGray, // Subtle UI elements
            accent: Color::Indexed(4),   // ANSI blue (folders, borders, headers)
            success: Color::Indexed(2),  // ANSI green (added, staged)
            warning: Color::Indexed(3),  // ANSI yellow (modified)
            error: Color::Indexed(1),    // ANSI red (deleted)
            info: Color::Indexed(6),     // ANSI cyan (renamed)
        }
    }
}

/// Flexible color value that can be ANSI index, named color, or hex
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ColorValue {
    Indexed(u8),
    Named(String),
}

impl ColorValue {
    pub fn to_color(&self) -> Color {
        match self {
            ColorValue::Indexed(n) => Color::Indexed(*n),
            ColorValue::Named(name) => match name.to_lowercase().as_str() {
                "black" => Color::Black,
                "red" => Color::Red,
                "green" => Color::Green,
                "yellow" => Color::Yellow,
                "blue" => Color::Blue,
                "magenta" => Color::Magenta,
                "cyan" => Color::Cyan,
                "white" => Color::White,
                "gray" | "grey" => Color::Gray,
                "darkgray" | "darkgrey" => Color::DarkGray,
                "lightred" => Color::LightRed,
                "lightgreen" => Color::LightGreen,
                "lightyellow" => Color::LightYellow,
                "lightblue" => Color::LightBlue,
                "lightmagenta" => Color::LightMagenta,
                "lightcyan" => Color::LightCyan,
                s if s.starts_with('#') && s.len() == 7 => {
                    parse_hex_color(s).unwrap_or(Color::Reset)
                }
                _ => Color::Reset,
            },
        }
    }
}

fn parse_hex_color(s: &str) -> Option<Color> {
    let r = u8::from_str_radix(&s[1..3], 16).ok()?;
    let g = u8::from_str_radix(&s[3..5], 16).ok()?;
    let b = u8::from_str_radix(&s[5..7], 16).ok()?;
    Some(Color::Rgb(r, g, b))
}

/// Raw layout config with optional fields for merging
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct RawLayoutConfig {
    mode: Option<LayoutMode>,
    max_rows: Option<u16>,
}

/// Raw config as parsed from TOML (uses Option for merge semantics)
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct RawConfig {
    delta: Option<DeltaConfig>,
    colors: Option<RawColorConfig>,
    layout: Option<RawLayoutConfig>,
}

/// Raw color config with optional fields for merging
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct RawColorConfig {
    text: Option<ColorValue>,
    text_muted: Option<ColorValue>,
    accent: Option<ColorValue>,
    success: Option<ColorValue>,
    warning: Option<ColorValue>,
    error: Option<ColorValue>,
    info: Option<ColorValue>,
}

impl Config {
    /// Load configuration, merging global and local configs
    pub fn load(repo_path: &Path) -> Self {
        let mut config = Config::default();

        // Load global config
        if let Some(global_path) = Self::global_config_path()
            && let Ok(raw) = Self::load_file(&global_path)
        {
            config.merge(raw);
        }

        // Load local config (overrides global)
        let local_path = repo_path.join(".kibitz.toml");
        if let Ok(raw) = Self::load_file(&local_path) {
            config.merge(raw);
        }

        config
    }

    fn global_config_path() -> Option<std::path::PathBuf> {
        dirs::config_dir().map(|p| p.join("kibitz").join("config.toml"))
    }

    fn load_file(path: &Path) -> Result<RawConfig, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&contents)?)
    }

    fn merge(&mut self, raw: RawConfig) {
        if let Some(delta) = raw.delta
            && delta.args.is_some()
        {
            self.delta.args = delta.args;
        }

        if let Some(colors) = raw.colors {
            let apply = |target: &mut Color, value: Option<ColorValue>| {
                if let Some(v) = value {
                    *target = v.to_color();
                }
            };
            apply(&mut self.colors.text, colors.text);
            apply(&mut self.colors.text_muted, colors.text_muted);
            apply(&mut self.colors.accent, colors.accent);
            apply(&mut self.colors.success, colors.success);
            apply(&mut self.colors.warning, colors.warning);
            apply(&mut self.colors.error, colors.error);
            apply(&mut self.colors.info, colors.info);
        }

        if let Some(layout) = raw.layout {
            if let Some(mode) = layout.mode {
                self.layout.mode = mode;
            }
            if let Some(max_rows) = layout.max_rows {
                self.layout.max_rows = max_rows;
            }
        }
    }
}
