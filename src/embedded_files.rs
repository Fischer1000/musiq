pub static INDEX_HTML: &'static [u8] = include_bytes!(concat!(env!("OUT_DIR"), "/index.html"));
pub static STYLES_CSS: &'static [u8] = include_bytes!(concat!(env!("OUT_DIR"), "/styles.css"));
pub static SCRIPT_JS: &'static [u8] = include_bytes!(concat!(env!("OUT_DIR"), "/script.js"));
pub static FAVICON_SVG: &'static [u8] = include_bytes!(concat!(env!("OUT_DIR"), "/favicon.svg"));


pub static CONFIG_MUSIQ: &'static [u8] =
    b"MUSIQ\nT\x38\x3A\x43\x44\x4D\x4F\x58\x5A\x63\x65\x6E\x6F\x78\x7E\x86\x88\xFF\xFF\xFF\xFF\xFFO\x02";