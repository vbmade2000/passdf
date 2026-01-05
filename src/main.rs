use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Password file with a passwords on each new line
    #[arg(short, long, value_name = "FILE")]
    passord_file: String,

    /// Single PDF file to process
    #[arg(
        short = 'f',
        long,
        value_name = "FILE",
        required_unless_present = "directory",
        conflicts_with = "directory"
    )]
    pdf_file: Option<String>,

    /// Directory containing PDF files to process
    #[arg(short, long, value_name = "DIR", conflicts_with = "pdf_file")]
    directory: Option<String>,

    /// Scan directory recursively to process every PDF file
    #[arg(short, long, requires = "directory", default_value_t = false)]
    recursive: bool,

    /// Show on screen a name of pdf file and a password used to open it  
    #[arg(short, long, default_value_t = false)]
    show_password: bool,
}

fn read_password_file(filename: &str) -> Vec<String> {
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);

    reader
        .lines()
        .map_while(|line| line.ok())
        .filter(|line| !line.is_empty())
        .collect()
}

fn main() {
    let args = Args::parse();

    let passwords = read_password_file(&args.passord_file);

    println!("Passwords: {:?}", passwords);
}
