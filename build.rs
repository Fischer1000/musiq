#![allow(unused_mut)]

#[cfg(feature = "use-encoding")]
use brotli;
#[cfg(feature = "use-encoding")]
use std::io::{Read, Write};

#[cfg(feature = "use-encoding")]
const BUF_SIZE: usize = 4096;
#[cfg(feature = "use-encoding")]
const COMP_QUALITY: u32 = 11;
#[cfg(feature = "use-encoding")]
const LG_WINDOW_SIZE: u32 = 21;

fn main() {
    let out_dir = std::env::var("OUT_DIR")
        .expect("No output directory environment variable set");

    'file_embedding: {
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
                let mut file_in = std::fs::File::open(file_in).expect("Input file cannot be opened");
                let mut input = brotli::CompressorReader::new(file_in, BUF_SIZE, COMP_QUALITY, LG_WINDOW_SIZE);

                let mut buf = Vec::new();
                input.read_to_end(&mut buf).expect("Error during input file reading");

                let file_out = std::fs::File::create(file_out).expect("Output file cannot be opened");
                file_out.write_all(&buf).expect("Error during output file writing");
            }
            #[cfg(not(feature = "use-encoding"))]
            '_plain: {
                std::fs::copy(file_in, file_out).expect("To be embedded file cannot be copied");
            }
        }
    }

    'code_generation: {
        // The contents of the to-be generated.rs file
        let mut gen_rs_content = String::new();

        let target_volume = std::env::var("TARGET_VOLUME")
            .unwrap_or("0.01".to_string())
            .parse::<f32>()
            .expect("TARGET_VOLUME could not be parsed as a float");

        if target_volume < -1.0 || target_volume > 1.0 {
            panic!("TARGET_VOLUME must be in range [-1, 1].");
        }

        gen_rs_content.push_str(&format!(
            "/// The target loudness of the normalized songs\npub const TARGET_VOLUME: f32 = {};\n",
            target_volume
        ));

        std::fs::write(format!("{out_dir}/generated.rs"), gen_rs_content).expect("Failed to write to generated.rs");
    }
}