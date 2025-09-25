use brotli;
use std::io::{Read, Write};
use flate2;

const BUF_SIZE: usize = 4096;
const COMP_QUALITY: u32 = 11;
const LG_WINDOW_SIZE: u32 = 21;

fn main() {
    let out_dir = std::env::var("OUT_DIR")
        .expect("No output directory environment variable set");

    let encoding = std::env::var("ENCODING").ok();
    let encoding_variant: &'static str;

    let input_files = [
        "res/webpage/index.html",
        "res/webpage/styles.css",
        "res/webpage/script.js",
        "res/webpage/favicon.svg"
    ];

    let output_files = [
        (format!("{out_dir}/index.html.bin"), "INDEX_HTML"),
        (format!("{out_dir}/styles.css.bin"), "STYLES_CSS"),
        (format!("{out_dir}/script.js.bin"), "SCRIPT_JS"),
        (format!("{out_dir}/favicon.svg.bin"), "FAVICON_SVG"),
    ];

    #[allow(unused_labels)]
    '_file_embedding: {
        let file_iter = input_files.iter().zip(output_files.iter());

        match encoding.as_deref() {
            Some("none") => {
                for (file_in, (file_out, _)) in file_iter {
                    std::fs::copy(file_in, file_out).expect("To be embedded file cannot be copied");
                }

                encoding_variant = "None";
            }
            Some("brotli") => {
                for (file_in, (file_out, _)) in file_iter {
                    let file_in = std::fs::File::open(file_in).expect("Input file cannot be opened");
                    let mut input = brotli::CompressorReader::new(
                        file_in,
                        BUF_SIZE,
                        COMP_QUALITY,
                        LG_WINDOW_SIZE
                    );

                    let mut buf = Vec::new();
                    input.read_to_end(&mut buf).expect("Error during input file reading");

                    let mut file_out = std::fs::File::create(file_out).expect("Output file cannot be opened");
                    file_out.write_all(&buf).expect("Error during output file writing");
                }

                encoding_variant = "Brotli";
            },
            None | Some("gzip") => {
                for (file_in, (file_out, _)) in file_iter {
                    let file_in = std::fs::File::open(file_in).expect("Input file cannot be opened");
                    let mut input = flate2::read::GzEncoder::new(&file_in, flate2::Compression::best());

                    let mut buf = Vec::new();
                    input.read_to_end(&mut buf).expect("Error during input file reading");

                    let mut file_out = std::fs::File::create(file_out).expect("Output file cannot be opened");
                    file_out.write_all(&buf).expect("Error during output file writing");
                }

                encoding_variant = "Gzip";
            }
            Some(_) => panic!("Unsupported file encoding")
        }
    }

    #[allow(unused_labels)]
    '_code_generation: {
        // The contents of the to-be generated.rs file
        let mut gen_rs_content = String::new();

        '_embedded_files: {
            let mut buf = "pub mod embedded_files {\n".to_string();

            for (path, name) in output_files.iter() {
                buf.push_str(
                    &format!(
                        "\tpub static {}: &'static [u8] = include_bytes!(\"{}\");\n",
                        name,
                        path.escape_default()
                    )
                )
            }

            buf.push_str("}\n");

            gen_rs_content.push_str(&buf);
        }

        '_target_volume: {
            let target_volume = std::env::var("TARGET_VOLUME")
                .unwrap_or("0.1".to_string())
                .parse::<f32>()
                .expect("TARGET_VOLUME could not be parsed as a float");

            if target_volume < -1.0 || target_volume > 1.0 {
                panic!("TARGET_VOLUME must be in range [-1, 1].");
            }

            gen_rs_content.push_str(&format!(
                "/// The target loudness of the normalized songs\npub const TARGET_VOLUME: f32 = {};\n",
                target_volume
            ));
        }

        '_encoding: {
            gen_rs_content.push_str(&format!(
                "/// The encoding of the embedded files\npub const ENCODING: Encoding = Encoding::{};\n",
                encoding_variant
            ));
        }

        std::fs::write(format!("{out_dir}/generated.rs"), gen_rs_content).expect("Failed to write to generated.rs");
    }
}