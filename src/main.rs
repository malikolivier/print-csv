extern crate clap;
extern crate csv;

use std::error::Error;
use std::fs::File;
use std::io;
use std::process;

use clap::{App, Arg};

fn main() -> Result<(), Box<Error>> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(Arg::with_name("INPUT").help("Sets the csv file to read"))
        .get_matches();

    if let Some(input_file) = matches.value_of("INPUT") {
        match File::open(input_file) {
            Ok(mut f) => read_csv(&mut f),
            Err(e) => {
                eprintln!("Error openign file '{}': {}", input_file, e);
                process::exit(1)
            }
        }
    } else {
        let mut stdin = io::stdin();
        let mut handle = stdin.lock();
        read_csv(&mut handle)
    }
}

fn read_csv<R: io::Read>(buf: &mut R) -> Result<(), Box<Error>> {
    let mut rdr = csv::Reader::from_reader(buf);
    for result in rdr.records() {
        // The iterator yields Result<StringRecord, Error>, so we check the
        // error here.
        let record = result?;
        println!("{:?}", record);
    }
    Ok(())
}
