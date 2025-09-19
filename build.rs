#![allow(unused_mut)]

#[cfg(feature = "use-encoding")]
use brotli;
#[cfg(feature = "use-encoding")]
use std::io::{Read, Write};
use std::env;

#[cfg(feature = "use-encoding")]
const BUF_SIZE: usize = 4096;
#[cfg(feature = "use-encoding")]
const COMP_QUALITY: u32 = 11;
#[cfg(feature = "use-encoding")]
const LG_WINDOW_SIZE: u32 = 21;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    let mut input_files = [
        "res/webpage/index.html",
        "res/webpage/styles.css",
        "res/webpage/script.js",
        "res/webpage/favicon.svg"
    ];

    let mut output_files = [
        format!("{out_dir}/index.html"),
        format!("{out_dir}/styles.css"),
        format!("{out_dir}/script.js"),
        format!("{out_dir}/favicon.svg")
    ];

    for (file_in, file_out) in input_files.into_iter().zip(output_files.into_iter()) {
        #[cfg(feature = "use-encoding")]
        '_brotli: {
            let mut file_in = std::fs::File::open(file_in).unwrap();
            let mut input = brotli::CompressorReader::new(file_in, BUF_SIZE, COMP_QUALITY, LG_WINDOW_SIZE);

            let mut buf = Vec::new();
            input.read_to_end(&mut buf).unwrap();

            let file_out = std::fs::File::create(file_out).unwrap();
            file_out.write_all(&buf).unwrap();
        }
        #[cfg(not(feature = "use-encoding"))]
        '_plain: {
            std::fs::copy(file_in, file_out).unwrap();
        }
    }
}