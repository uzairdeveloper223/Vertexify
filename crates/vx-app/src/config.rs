#![allow(dead_code)]
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub render: RenderConfig,
    #[serde(default)]
    pub vx: VxConfig,
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub io: IoConfig,
}

#[derive(Debug, Deserialize)]
pub struct RenderConfig {
    #[serde(default = "defaults::render_scale")]
    pub render_scale: f32,
    #[serde(default = "defaults::idle_rendering")]
    pub idle_rendering: bool,
    #[serde(default = "defaults::msaa_samples")]
    pub msaa_samples: u32,
    #[serde(default = "defaults::ssao")]
    pub ssao: bool,
    #[serde(default = "defaults::ssao_samples")]
    pub ssao_samples: u32,
    #[serde(default = "defaults::max_lights")]
    pub max_lights: u32,
}

#[derive(Debug, Deserialize)]
pub struct VxConfig {
    #[serde(default = "defaults::eval_timeout_s")]
    pub eval_timeout_s: u64,
    #[serde(default = "defaults::max_recursion_depth")]
    pub max_recursion_depth: u32,
    #[serde(default = "defaults::max_iterations")]
    pub max_iterations: u64,
    #[serde(default = "defaults::max_triangle_count")]
    pub max_triangle_count: u64,
}

#[derive(Debug, Deserialize)]
pub struct UiConfig {
    #[serde(default = "defaults::font_size")]
    pub font_size: u32,
    #[serde(default = "defaults::theme")]
    pub theme: String,
    #[serde(default = "defaults::autosave_interval_s")]
    pub autosave_interval_s: u64,
    #[serde(default = "defaults::undo_history")]
    pub undo_history: u32,
}

#[derive(Debug, Deserialize)]
pub struct IoConfig {
    #[serde(default = "defaults::default_export_format")]
    pub default_export_format: String,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            render_scale: defaults::render_scale(),
            idle_rendering: defaults::idle_rendering(),
            msaa_samples: defaults::msaa_samples(),
            ssao: defaults::ssao(),
            ssao_samples: defaults::ssao_samples(),
            max_lights: defaults::max_lights(),
        }
    }
}

impl Default for VxConfig {
    fn default() -> Self {
        Self {
            eval_timeout_s: defaults::eval_timeout_s(),
            max_recursion_depth: defaults::max_recursion_depth(),
            max_iterations: defaults::max_iterations(),
            max_triangle_count: defaults::max_triangle_count(),
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            font_size: defaults::font_size(),
            theme: defaults::theme(),
            autosave_interval_s: defaults::autosave_interval_s(),
            undo_history: defaults::undo_history(),
        }
    }
}

impl Default for IoConfig {
    fn default() -> Self {
        Self {
            default_export_format: defaults::default_export_format(),
        }
    }
}



mod defaults {
    pub fn render_scale() -> f32 {
        1.0
    }
    pub fn idle_rendering() -> bool {
        true
    }
    pub fn msaa_samples() -> u32 {
        1
    }
    pub fn ssao() -> bool {
        true
    }
    pub fn ssao_samples() -> u32 {
        8
    }
    pub fn max_lights() -> u32 {
        16
    }
    pub fn eval_timeout_s() -> u64 {
        30
    }
    pub fn max_recursion_depth() -> u32 {
        64
    }
    pub fn max_iterations() -> u64 {
        1_000_000
    }
    pub fn max_triangle_count() -> u64 {
        10_000_000
    }
    pub fn font_size() -> u32 {
        14
    }
    pub fn theme() -> String {
        "dark".to_string()
    }
    pub fn autosave_interval_s() -> u64 {
        30
    }
    pub fn undo_history() -> u32 {
        100
    }
    pub fn default_export_format() -> String {
        "obj".to_string()
    }
}

pub fn load() -> Config {
    match std::fs::read_to_string(config_path()) {
        Ok(text) => toml::from_str(&text).unwrap_or_default(),
        Err(_) => Config::default(),
    }
}

fn config_path() -> PathBuf {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let mut home = std::env::var_os("HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("/tmp"));
            home.push(".config");
            home
        });
    base.join("vertexify").join("vertexify.toml")
}
