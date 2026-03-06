use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Mode {
    pub width: i32,
    pub height: i32,
    pub refresh_rate: f32,
}

#[derive(Debug, Clone)]
pub struct Monitor {
    pub name: String,
    pub description: String,
    pub width: i32,
    pub height: i32,
    pub refresh_rate: f32,
    pub x: i32,
    pub y: i32,
    pub scale: f32,
    pub focused: bool,
    pub enabled: bool,
    pub available_modes: Vec<Mode>,
}

pub trait Backend {
    fn list_monitors(&self) -> Result<Vec<Monitor>>;
    fn set_monitor_mode(&self, monitor: &str, width: i32, height: i32, refresh: f32) -> Result<()>;
    fn enable_monitor(&self, monitor: &str) -> Result<()>;
    fn disable_monitor(&self, monitor: &str) -> Result<()>;
    fn name(&self) -> &'static str;
}
