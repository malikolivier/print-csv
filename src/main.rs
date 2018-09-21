extern crate clap;
extern crate csv;
extern crate libc;
extern crate unicode_width;

use std::cmp;
use std::error::Error;
use std::fs::File;
use std::io;
use std::process::{self, Command, Stdio};

use clap::{App, Arg};
use unicode_width::UnicodeWidthChar;

fn main() -> Result<(), Box<Error>> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(Arg::with_name("INPUT").help("Sets the csv file to read"))
        .get_matches();

    if let Some(input_file) = matches.value_of("INPUT") {
        match File::open(input_file) {
            Ok(mut f) => run(&mut f),
            Err(e) => {
                eprintln!("Error opening file '{}': {}", input_file, e);
                process::exit(1)
            }
        }
    } else {
        let istty = unsafe { libc::isatty(libc::STDIN_FILENO as i32) } != 0;
        if istty {
            eprintln!("{}", matches.usage());
            process::exit(1)
        } else {
            let mut stdin = io::stdin();
            let mut handle = stdin.lock();
            run(&mut handle)
        }
    }
}

fn run<R: io::Read>(input: &mut R) -> Result<(), Box<Error>> {
    let istty = unsafe { libc::isatty(libc::STDOUT_FILENO as i32) } != 0;
    if istty {
        let mut less = Command::new("less")
            .stdin(Stdio::piped())
            .arg("-S")
            .spawn()
            .expect("Could not run less!");

        {
            let less_in = less.stdin.as_mut().unwrap();
            read_csv(input, less_in)?;
        }

        less.wait().expect("Failed to wait on child");
    } else {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        read_csv(input, &mut handle)?;
    }
    Ok(())
}

fn read_csv<R: io::Read, W: io::Write>(buf: &mut R, out: &mut W) -> Result<(), Box<Error>> {
    let mut rdr = csv::Reader::from_reader(buf);

    let headers = rdr.headers()?.clone();
    let mut cols: Vec<_> = headers.iter().map(|header| header.len()).collect();

    const BUFFER_SIZE: usize = 3000;
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

    let mut out_buf = io::BufWriter::new(out);

    write_record(&headers, &cols, &mut out_buf)?;
    for record in buffer {
        write_record(&record, &cols, &mut out_buf)?;
    }

    for result in rdr.records() {
        let record = result?;

        write_record(&record, &cols, &mut out_buf)?;
    }
    Ok(())
}

fn write_record<W: io::Write>(
    record: &csv::StringRecord,
    cols: &[usize],
    out: &mut W,
) -> Result<(), Box<Error>> {
    for (i, field) in record.iter().enumerate() {
        write!(out, "\"")?;
        for c in field.chars() {
            if c == '\t' {
                write!(out, "    ")?;
            } else {
                write!(out, "{}", c)?;
            }
        }
        write!(out, "\"")?;
        for _ in terminal_length(&field)..(cols[i] + 2) {
            write!(out, " ")?;
        }
    }
    writeln!(out)?;
    Ok(())
}

fn terminal_length(string: &str) -> usize {
    string.chars().fold(0, |acc, c| {
        acc + if c == '\t' {
            4
        } else {
            UnicodeWidthChar::width(c).unwrap_or(0)
        }
    })
}
