#![feature(plugin)]
#![plugin(docopt_macros)]

extern crate rustc_serialize;
extern crate docopt;
extern crate libc;
extern crate image;

mod fsextra;
use fsextra::{ Stdin, Stdout, Reopen };

use image::*;

#[derive(Debug, RustcDecodable)]
enum Format { PNG, JPEG, GIF, WEBP, PPM, TIFF, TGA, BMP }

docopt!(Args, "
Usage:
    polly-rs [-i INPUT] -x FORMAT [ -o OUTPUT] [-p] <SIZE>
    polly-rs (-h | --help)

Options:
    -h, --help      Show this message
    -i INPUT        Input file (default stdin)
    -o OUTPUT       Output file (default stdout)
    -x FORMAT       Input format
    -p              Invert the colors",
flag_i: Option<String>,
flag_o: Option<String>,
flag_x: Format,
flag_p: bool,
flag_h: bool,
flag_v: bool,
arg_SIZE: u32);
const LO : char = 32  as char;
const LM : char = 46  as char;
const HM : char = 58  as char;
const HI : char = 120 as char;
const INV : f64 = (HI as u8 - LO as u8) as f64/HI as u8 as f64 * 255.0;

fn main() {
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());
    let mut qin = Stdin::own();
    let mut qout = Stdout::own();
    if let Some(ref input) = args.flag_i {
        qin.reopen(input).unwrap();
    }
    if let Some(ref output) = args.flag_o {
        qout.reopen(output).unwrap();
    }
    let format = match args.flag_x {
        Format::PNG  => ImageFormat::PNG,
        Format::JPEG => ImageFormat::JPEG,
        Format::GIF  => ImageFormat::GIF,
        Format::WEBP => ImageFormat::WEBP,
        Format::PPM  => ImageFormat::PPM,
        Format::TIFF => ImageFormat::TIFF,
        Format::TGA  => ImageFormat::TGA,
        Format::BMP  => ImageFormat::BMP,
    };
    if args.arg_SIZE != 0 {
        let nw : u32; let nh: u32;
        let gray = {
            let img = image::load(qin.file, format).unwrap();
            nw = args.arg_SIZE + 1;
            nh = (nw as f64*(img.height() as f64/img.width() as f64)) as u32;
            let buf = img.resize(nw, nh, Lanczos3);
            //NOTE: a little bit of contrast wouldn't hurt
            buf.to_luma()
        };
        println!("P5 {} {} {}", nw, nh, HI as u8);
        for (x, _, px) in gray.enumerate_pixels() {
            if x == 0 {
                println!("");
            } else {
                let col = if args.flag_p {
                    INV - px.data[0] as f64
                } else {
                    px.data[0] as f64
                };
                let ch = if col                >= 123.0 { HI }
                    else if col < 123.0 && col >= 98.0  { HM }
                    else if col <  98.0 && col >= 68.0  { LM }
                    else                                { LO };
                print!("{}", ch);
            }
        }
        println!("");
    } else {
        panic!("SIZE cannot be 0");
    }
}
