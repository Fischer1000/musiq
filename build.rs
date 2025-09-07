use brotli;
use std::env;
use std::io::{Read, Write};

const BUF_SIZE: usize = 4096;
const COMP_QUALITY: u32 = 11;
const LG_WINDOW_SIZE: u32 = 21;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    let mut input_files = [
        std::fs::File::open("res/webpage/index.html").unwrap(),
        std::fs::File::open("res/webpage/styles.css").unwrap(),
        std::fs::File::open("res/webpage/script.js").unwrap(),
        std::fs::File::open("res/webpage/favicon.svg").unwrap()
    ];

    let mut output_files = [
        std::fs::File::create(format!("{out_dir}/index.html.br")).unwrap(),
        std::fs::File::create(format!("{out_dir}/styles.css.br")).unwrap(),
        std::fs::File::create(format!("{out_dir}/script.js.br")).unwrap(),
        std::fs::File::create(format!("{out_dir}/favicon.svg.br")).unwrap()
    ];

    for (file_in, file_out) in input_files.iter_mut().zip(output_files.iter_mut()) {
        let mut input = brotli::CompressorReader::new(file_in, BUF_SIZE, COMP_QUALITY, LG_WINDOW_SIZE);

        let mut buf = Vec::new();
        input.read_to_end(&mut buf).unwrap();

        file_out.write_all(&buf).unwrap();
    }

    println!("cargo::rerun-if-changed=build.rs");
}