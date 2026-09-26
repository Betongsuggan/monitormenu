use anyhow::Result;

#[derive(Debug, Clone, PartialEq)]
pub struct Mode {
    pub width: i32,
    pub height: i32,
    pub refresh_rate: f32,
}

/// Everything the menu can change about an output. Backends read it from the
/// compositor and apply a whole new one, so changing one field never resets
/// the others.
#[derive(Debug, Clone, PartialEq)]
pub struct OutputConfig {
    pub enabled: bool,
    pub mode: Mode,
    pub x: i32,
    pub y: i32,
    pub scale: f32,
    /// wl_output transform: 0-3 rotate by 0/90/180/270 degrees, 4-7 flipped
    pub transform: u8,
    pub vrr: bool,
    pub hdr: bool,
    pub bitdepth: Option<u8>,
    /// Kept as they are when applying (set in the Nix config, not the menu)
    pub sdr_brightness: Option<f32>,
    pub sdr_saturation: Option<f32>,
    /// The output this one mirrors
    pub mirror_of: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Monitor {
    pub name: String,
    pub description: String,
    pub focused: bool,
    pub config: OutputConfig,
    pub available_modes: Vec<Mode>,
}

impl Monitor {
    /// Size in the global layout: the mode, rotated, divided by the scale
    pub fn logical_size(&self) -> (i32, i32) {
        let c = &self.config;
        let (w, h) = if c.transform % 2 == 1 {
            (c.mode.height, c.mode.width)
        } else {
            (c.mode.width, c.mode.height)
        };
        let scale = if c.scale > 0.0 { c.scale } else { 1.0 };
        (
            (w as f32 / scale).round() as i32,
            (h as f32 / scale).round() as i32,
        )
    }
}

/// What a compositor can change at runtime; the menu offers nothing else
#[derive(Debug, Clone, Copy)]
pub struct Capabilities {
    pub hdr: bool,
    pub mirror: bool,
}

pub trait Backend {
    fn list_monitors(&self) -> Result<Vec<Monitor>>;
    /// Make `monitor` match `config`
    fn apply(&self, monitor: &Monitor, config: &OutputConfig) -> Result<()>;
    fn capabilities(&self) -> Capabilities;
    fn name(&self) -> &'static str;
}
