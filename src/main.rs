extern crate clap;
extern crate csv;
extern crate icu_sys;

use std::cmp;
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

    let headers = rdr.headers()?.clone();
    let mut cols: Vec<_> = headers.iter().map(|header| header.len()).collect();

    const BUFFER_SIZE: usize = 1000;
    let mut buffer = Vec::with_capacity(BUFFER_SIZE);

    for result in rdr.records().take(BUFFER_SIZE) {
        let record = result?;
        while cols.len() < record.len() {
            cols.push(0);
        }

        for (i, field) in record.iter().enumerate() {
            cols[i] = cmp::max(cols[i], terminal_length(&field));
        }

        buffer.push(record.clone());
    }

    write_record(&headers, &cols);
    for record in buffer {
        write_record(&record, &cols);
    }

    for result in rdr.records() {
        let record = result?;

        write_record(&record, &cols);
    }
    Ok(())
}

fn write_record(record: &csv::StringRecord, cols: &[usize]) {
    for (i, field) in record.iter().enumerate() {
        print!("\"{}\"", field);
        for _ in terminal_length(&field)..(cols[i] + 2) {
            print!(" ");
        }
    }
    println!("");
}

fn terminal_length(string: &str) -> usize {
    string
        .chars()
        .fold(0, |acc, c| acc + if is_fullwidth(c) { 2 } else { 1 })
}

/// https://stackoverflow.com/questions/15114303/determine-whether-a-unicode-character-is-fullwidth-or-halfwidth-in-c
/// bool is_fullwidth(UChar32 c) {
///     int width = u_getIntPropertyValue(c, UCHAR_EAST_ASIAN_WIDTH);
///     return width == U_EA_FULLWIDTH || width == U_EA_WIDE;
/// }
fn is_fullwidth(c: char) -> bool {
    let width = unsafe {
        icu_sys::uchar::u_getIntPropertyValue_52(c as i32, icu_sys::uchar::UCHAR_EAST_ASIAN_WIDTH)
    } as u32;
    width == icu_sys::uchar::U_EA_FULLWIDTH || width == icu_sys::uchar::U_EA_WIDE
}
