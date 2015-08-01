#![feature(plugin)]
#![plugin(docopt_macros)]

extern crate rustc_serialize;
extern crate docopt;
extern crate libc;
extern crate image;

use image::*;
use std::ffi::CString;
use libc::funcs::posix88::fcntl::open as libcopen;
use libc::funcs::posix88::unistd::dup2;
use libc::consts::os::posix88::{ O_RDONLY, O_WRONLY, O_CREAT };
use std::io::{ Error, Result };
use std::os::unix::io::{ AsRawFd, RawFd, FromRawFd };
use std::os::unix::fs::PermissionsExt;
use std::os::unix::fs::MetadataExt;
use std::fs::{ File, Metadata };

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

struct Stdin  { file: File }
struct Stdout { file: File }
struct Stderr { file: File }

impl  AsRawFd for Stdin  { fn as_raw_fd(&self) -> RawFd { 0 } }
impl  AsRawFd for Stdout { fn as_raw_fd(&self) -> RawFd { 1 } }
impl  AsRawFd for Stderr { fn as_raw_fd(&self) -> RawFd { 2 } }

trait FileInfo { fn metadata(&self) -> Result<Metadata>; }

impl FileInfo for Stdin {
    fn metadata(&self) -> Result<Metadata> { self.file.metadata() }
}

impl FileInfo for Stdout {
    fn metadata(&self) -> Result<Metadata> { self.file.metadata() }
}

impl FileInfo for Stderr {
    fn metadata(&self) -> Result<Metadata> { self.file.metadata() }
}

impl Stdin {
    fn own() -> Stdin  { unsafe { Stdin  { file: File::from_raw_fd(0) } } }
}
impl Stdout {
    fn own() -> Stdout { unsafe { Stdout { file: File::from_raw_fd(1) } } }
}
impl Stderr {
    fn own() -> Stderr { unsafe { Stderr { file: File::from_raw_fd(2) } } }
}

trait ReopenMode {
    fn oreopen(&mut self, path: &String, mode: i32) -> Result<()>;
}

impl<T> ReopenMode for T where T : AsRawFd + FileInfo {
    fn oreopen(&mut self, path: &String, mode: i32) -> Result<()> {
        let fd = self.as_raw_fd();
        let cpath = CString::new(&path[..]).unwrap();
        let metadata = try!(self.metadata());
        let file = unsafe { libcopen(cpath.as_ptr(), mode, metadata.mode()) };
        if file == -1 {
            return Err(Error::last_os_error());
        }
        if unsafe { dup2(file, fd) == -1 } {
            return Err(Error::last_os_error());
        }
        Ok(())
    }
}

trait Reopen : ReopenMode {
    fn reopen(&mut self, path: &String) -> Result<()>;
}

impl Reopen for Stdin {
    fn reopen(&mut self, path: &String) -> Result<()> {
        self.oreopen(path, O_RDONLY)
    }
}

impl Reopen for Stdout {
    fn reopen(&mut self, path: &String) -> Result<()> {
        self.oreopen(path, O_WRONLY|O_CREAT)
    }
}

impl Reopen for Stderr {
    fn reopen(&mut self, path: &String) -> Result<()> {
        self.oreopen(path, O_WRONLY|O_CREAT)
    }
}

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
            //NOTE: a little bit of constast wouldn't hurt
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
