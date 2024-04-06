//! This binary can be used to quickly run the bencode parser
//! `cargo run --bin ben_parser -- --help`
//!
//! `--file-path <path to bencode file>` to use file
//!
//! `--ben-string <benstring>` where example <benstring> is i32e
//!
//! `--write-to <filepath>` path to write file to else to stdout

use clap::{command, Parser};
use std::{fs, path::PathBuf};
use xtorrent::bencode_parser::{BenStruct, BencodeParser};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    // use file
    #[arg(short, long, value_name = "FILE")]
    file_name: Option<PathBuf>,
    // ben string if no file
    #[arg(short, long, value_name = "STRING")]
    ben_string: Option<String>,
    #[arg(short, long, value_name = "FILE_W")]
    write_to: Option<PathBuf>,
}

// #Panics
fn main() {
    println!("The bencode parser runtime");

    let env_vars = Args::parse();
    let mut bc_parser: BencodeParser;

    if let Some(file_path) = env_vars.file_name.as_deref() {
        println!("File path specified {}", file_path.display());
        bc_parser = BencodeParser::new_w_file(file_path);
    } else if let Some(ben_string) = env_vars.ben_string {
        bc_parser = BencodeParser::new_w_string(ben_string);
    } else {
        panic!("Use either file path for bencode or direct ben string with --ben-string option")
    };

    if let Some(w_file_path) = env_vars.write_to.as_deref() {
        println!("Writing file to {}", w_file_path.display());
        let _ = fs::write(w_file_path, format!("{:#?}", bc_parser.decode_bencode()));
        println!("Parsed bencode written to {}", w_file_path.display());
    } else {
        println!("{:#?}", bc_parser.decode_bencode());
    }
}
