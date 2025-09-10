pub static INDEX_HTML: &'static [u8] = include_bytes!(concat!(env!("OUT_DIR"), "/index.html"));
pub static STYLES_CSS: &'static [u8] = include_bytes!(concat!(env!("OUT_DIR"), "/styles.css"));
pub static SCRIPT_JS: &'static [u8] = include_bytes!(concat!(env!("OUT_DIR"), "/script.js"));
pub static FAVICON_SVG: &'static [u8] = include_bytes!(concat!(env!("OUT_DIR"), "/favicon.svg"));


pub static CONFIG_MUSIQ: &'static [u8] = &crate::config::default_config_bytes();