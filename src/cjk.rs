// Please refer to the following pages:
// https://www.unicode.org/reports/tr11/
// https://www.unicode.org/Public/UCD/latest/ucd/EastAsianWidth.txt
const DB: &[u8] = include_bytes!("EastAsianWidth.txt");

use std::io::{self, BufRead};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum EastAsianWidth {
    A,
    F,
    H,
    N,
    Na,
    W,
}

const UTF8_MAX: usize = 0x11_0000;

lazy_static! {
    static ref RULES: [EastAsianWidth; UTF8_MAX] = {
        let mut ret = [EastAsianWidth::A; UTF8_MAX];

        let buf = io::Cursor::new(DB);
        for line in buf.lines() {
            let line = line.unwrap();

            let line = line.split('#').next().unwrap();
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let (code_points, value) = {
                let mut split = line.split(';');
                let code_points = split.next().unwrap();
                let value = split.next().expect("Missing value after ';'");
                (code_points, value)
            };

            let value = match value {
                "A" => EastAsianWidth::A,
                "F" => EastAsianWidth::F,
                "N" => EastAsianWidth::N,
                "H" => EastAsianWidth::H,
                "Na" => EastAsianWidth::Na,
                "W" => EastAsianWidth::W,
                _ => panic!("Unexpected EastAsianWidth value: {}", value),
            };

            if code_points.contains("..") {
                let mut split = code_points.split("..");
                let start = split.next().unwrap();
                let end = split.next().unwrap();
                let start = usize::from_str_radix(start, 16).unwrap();
                let end = usize::from_str_radix(end, 16).unwrap();
                for i in start..(end + 1) {
                    ret[i] = value;
                }
            } else {
                let code_point = usize::from_str_radix(code_points, 16).unwrap();
                ret[code_point] = value;
            }
        }

        ret
    };
}

pub fn is_fullwidth(c: char) -> bool {
    //let mut b = [0; 4];
    //c.encode_utf8(&mut b);
    //let uchar32: u32 = unsafe { ::std::mem::transmute(b) };
    //println!(
    //    "{} {:?} {} {} {}",
    //    c,
    //    b,
    //    uchar32,
    //    c.escape_unicode(),
    //    c as u32
    //);
    let width = RULES[(c as u32) as usize];
    width == EastAsianWidth::F || width == EastAsianWidth::W
}
