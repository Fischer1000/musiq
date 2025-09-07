pub static INDEX_HTML_BR: &'static [u8] = include_bytes!(concat!(env!("OUT_DIR"), "/index.html.br"));
pub static STYLES_CSS_BR: &'static [u8] = include_bytes!(concat!(env!("OUT_DIR"), "/styles.css.br"));
pub static SCRIPT_JS_BR: &'static [u8] = include_bytes!(concat!(env!("OUT_DIR"), "/script.js.br"));
pub static FAVICON_SVG_BR: &'static [u8] = include_bytes!(concat!(env!("OUT_DIR"), "/favicon.svg.br"));


pub static CONFIG_MUSIQ: &'static [u8] =
    b"MUSIQ\nT\x38\x3A\x43\x44\x4D\x4F\x58\x5A\x63\x65\x6E\x6F\x78\x7E\x86\x88\xFF\xFF\xFF\xFF\xFFO\x02";