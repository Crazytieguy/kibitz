use ratatui::style::Color;
use serde::Deserialize;
use std::path::Path;

/// Top-level configuration
#[derive(Debug, Clone, Default)]
pub struct Config {
    pub delta: DeltaConfig,
    pub colors: ColorConfig,
}

/// Delta pass-through configuration
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct DeltaConfig {
    /// Additional args appended to delta command
    /// Example: "--side-by-side --line-numbers"
    pub args: Option<String>,
}

/// File status colors configuration
#[derive(Debug, Clone)]
pub struct ColorConfig {
    pub folder: Color,
    pub modified: Color,
    pub added: Color,
    pub deleted: Color,
    pub renamed: Color,
    pub staged: Color,
    pub staged_modified: Color,
    pub untracked: Color,
}

impl Default for ColorConfig {
    fn default() -> Self {
        Self {
            folder: Color::Indexed(4),          // ANSI blue
            modified: Color::Indexed(3),        // ANSI yellow
            added: Color::Indexed(2),           // ANSI green
            deleted: Color::Indexed(1),         // ANSI red
            renamed: Color::Indexed(6),         // ANSI cyan
            staged: Color::Indexed(2),          // ANSI green
            staged_modified: Color::Indexed(5), // ANSI magenta
            untracked: Color::Indexed(8),       // ANSI bright black
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

/// Raw config as parsed from TOML (uses Option for merge semantics)
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct RawConfig {
    delta: Option<DeltaConfig>,
    colors: Option<RawColorConfig>,
}

/// Raw color config with optional fields for merging
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct RawColorConfig {
    folder: Option<ColorValue>,
    modified: Option<ColorValue>,
    added: Option<ColorValue>,
    deleted: Option<ColorValue>,
    renamed: Option<ColorValue>,
    staged: Option<ColorValue>,
    staged_modified: Option<ColorValue>,
    untracked: Option<ColorValue>,
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
        let local_path = repo_path.join(".git-diff-tui.toml");
        if let Ok(raw) = Self::load_file(&local_path) {
            config.merge(raw);
        }

        config
    }

    fn global_config_path() -> Option<std::path::PathBuf> {
        dirs::config_dir().map(|p| p.join("git-diff-tui").join("config.toml"))
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
            apply(&mut self.colors.folder, colors.folder);
            apply(&mut self.colors.modified, colors.modified);
            apply(&mut self.colors.added, colors.added);
            apply(&mut self.colors.deleted, colors.deleted);
            apply(&mut self.colors.renamed, colors.renamed);
            apply(&mut self.colors.staged, colors.staged);
            apply(&mut self.colors.staged_modified, colors.staged_modified);
            apply(&mut self.colors.untracked, colors.untracked);
        }
    }
}
