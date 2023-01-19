use std::path::Path;

use clap::{Arg, Command};
use schemars::schema_for;

use rose_file_readers::{
    editor_friendly::QsdFile, QsdReadOptions, RoseFile, RoseFileReader, RoseFileWriter,
};

fn main() {
    let command = Command::new("rose-conv")
        .about("ROSE file format converter")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("convert")
                .about("Convert ROSE file formats")
                .arg(
                    Arg::new("src")
                        .help("Source file path")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::new("dst")
                        .help("Destination file path")
                        .takes_value(true)
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("schema")
                .about("Generate a JSON schema for supported file formats")
                .arg(
                    Arg::new("file_type")
                        .help("Source file path")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::new("dst")
                        .help("Destination file path")
                        .takes_value(true)
                        .required(true),
                ),
        );
    let matches = command.get_matches();

    match matches.subcommand() {
        Some(("convert", sub_matches)) => {
            let src = Path::new(
                sub_matches
                    .get_one::<String>("src")
                    .map(|s| s.as_str())
                    .unwrap(),
            );
            let dst = Path::new(
                sub_matches
                    .get_one::<String>("dst")
                    .map(|s| s.as_str())
                    .unwrap(),
            );

            let src_extension = src
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.to_ascii_lowercase());
            let src_extension = src_extension.as_deref();
            let dst_extension = dst
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.to_ascii_lowercase());
            let dst_extension = dst_extension.as_deref();

            match src_extension {
                Some("qsd") => {
                    let Ok(data) = std::fs::read(src) else {
                        println!("Failed to read file {}", src.display());
                        return;
                    };
                    let qsd = match <QsdFile as RoseFile>::read(
                        RoseFileReader::from(&data),
                        &QsdReadOptions::default(),
                    ) {
                        Ok(qsd) => qsd,
                        Err(error) => {
                            println!("Failed to parse QsdFile {}: {}", src.display(), error);
                            return;
                        }
                    };

                    match dst_extension {
                        Some("yaml") => {
                            let mut yaml_bytes = Vec::with_capacity(1024 * 1024);
                            match serde_yaml::with::singleton_map_recursive::serialize(
                                &qsd,
                                &mut serde_yaml::Serializer::new(&mut yaml_bytes),
                            ) {
                                Ok(_) => {}
                                Err(error) => {
                                    println!("Failed to serialize yaml {}", error);
                                    return;
                                }
                            }
                            let yaml = String::from_utf8(yaml_bytes).unwrap();

                            if let Err(error) = std::fs::write(dst, yaml) {
                                println!("Failed to write to {}: {}", dst.display(), error);
                            }
                        }
                        Some("json") => {
                            let json = match serde_json::to_string(&qsd) {
                                Ok(json) => json,
                                Err(error) => {
                                    println!("Failed to serialize json {}", error);
                                    return;
                                }
                            };

                            if let Err(error) = std::fs::write(dst, json) {
                                println!("Failed to write to {}: {}", dst.display(), error);
                            }
                        }
                        _ => {
                            println!("Unknown file extension for dest {}", dst.display());
                        }
                    }
                }
                Some("yaml") => {
                    let Ok(data) = std::fs::read_to_string(src) else {
                        println!("Failed to read file {}", src.display());
                        return;
                    };

                    match dst_extension {
                        Some("qsd") => {
                            let qsd: QsdFile =
                                match serde_yaml::with::singleton_map_recursive::deserialize(
                                    serde_yaml::Deserializer::from_str(&data),
                                ) {
                                    Ok(qsd) => qsd,
                                    Err(error) => {
                                        println!(
                                            "Failed to parse QsdFile {}: {}",
                                            src.display(),
                                            error
                                        );
                                        return;
                                    }
                                };

                            let mut writer = RoseFileWriter::default();
                            if let Err(error) = qsd.write(&mut writer, &()) {
                                println!("Failed to serialise QsdFile: {}", error);
                                return;
                            }

                            if let Err(error) = std::fs::write(dst, &writer.buffer[..]) {
                                println!("Failed to write to {}: {}", dst.display(), error);
                            }
                        }
                        _ => {
                            println!("Unknown file extension for dest {}", dst.display());
                        }
                    }
                }
                _ => {
                    println!("Unknown file extension for source {}", src.display());
                }
            }
        }
        Some(("schema", sub_matches)) => {
            let file_type = sub_matches
                .get_one::<String>("file_type")
                .map(|s| s.as_str())
                .unwrap();
            let dst = Path::new(
                sub_matches
                    .get_one::<String>("dst")
                    .map(|s| s.as_str())
                    .unwrap(),
            );

            match file_type {
                "qsd" => {
                    let schema = schema_for!(QsdFile);
                    let json = match serde_json::to_string_pretty(&schema) {
                        Ok(json) => json,
                        Err(error) => {
                            println!("Failed to serialize schema json {}", error);
                            return;
                        }
                    };

                    if let Err(error) = std::fs::write(dst, json) {
                        println!("Failed to write to {}: {}", dst.display(), error);
                    }
                }
                _ => {
                    println!("Invalid schema file type {}", file_type);
                }
            }
        }
        _ => unimplemented!(),
    }
}
