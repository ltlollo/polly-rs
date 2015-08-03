
use libc::funcs::posix88::fcntl::open;
use libc::funcs::posix88::unistd::dup2;
use libc::consts::os::posix88::{O_RDONLY, O_WRONLY, O_CREAT, O_TRUNC};
use libc::consts::os::posix88::{STDIN_FILENO, STDOUT_FILENO, STDERR_FILENO};
use libc::funcs::c95::stdlib::exit;
use libc::consts::os::c95::EXIT_FAILURE;

use std::io::{Error, Result, Write};
use std::os::unix::io::{AsRawFd, RawFd, FromRawFd};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::fs::{File, Metadata};
use std::fmt::Display;

pub struct Stdin {
    pub file: File,
}
pub struct Stdout {
    pub file: File,
}
pub struct Stderr {
    pub file: File,
}

impl AsRawFd for Stdin  {    fn as_raw_fd(&self) -> RawFd {
        STDIN_FILENO
    } }
impl AsRawFd for Stdout {    fn as_raw_fd(&self) -> RawFd {
        STDOUT_FILENO
    } }
impl AsRawFd for Stderr {    fn as_raw_fd(&self) -> RawFd {
        STDERR_FILENO
    } }

trait FileInfo {    fn metadata(&self) -> Result<Metadata>; }

impl FileInfo for Stdin {
    fn metadata(&self) -> Result<Metadata> {
        self.file.metadata()
    }
}

impl FileInfo for Stdout {
    fn metadata(&self) -> Result<Metadata> {
        self.file.metadata()
    }
}

impl FileInfo for Stderr {
    fn metadata(&self) -> Result<Metadata> {
        self.file.metadata()
    }
}

impl Stdin {
    pub fn own() -> Stdin {
        unsafe {
            Stdin { file: File::from_raw_fd(STDIN_FILENO) }
        }
    }
}
impl Stdout {
    pub fn own() -> Stdout {
        unsafe {
            Stdout { file: File::from_raw_fd(STDOUT_FILENO) }
        }
    }
}
impl Stderr {
    pub fn own() -> Stderr {
        unsafe {
            Stderr { file: File::from_raw_fd(STDERR_FILENO) }
        }
    }
}

impl FileInfo for File {
    fn metadata(&self) -> Result<Metadata> {
        self.metadata()
    }
}

trait ReopenMode {
    fn oreopen(&mut self, path: &String, mode: i32) -> Result<()>;
}

impl<T> ReopenMode for T where T : AsRawFd + FileInfo {
    fn oreopen(&mut self, path: &String, mode: i32) -> Result<()> {
        let fd = self.as_raw_fd();
        let metadata = try!(self.metadata());
        let file = unsafe {
            open(path.as_ptr() as *const i8, mode, metadata.mode())
        };
        if file == -1 {
            return Err(Error::last_os_error());
        }
        if unsafe {
            dup2(file, fd) == -1
        } {
            return Err(Error::last_os_error());
        }
        Ok(())
    }
}

pub trait Reopen : ReopenMode {
    fn reopen(&mut self, path: &String) -> Result<()>;
}

impl Reopen for Stdin {
    fn reopen(&mut self, path: &String) -> Result<()> {
        self.oreopen(path, O_RDONLY)
    }
}

impl Reopen for Stdout {
    fn reopen(&mut self, path: &String) -> Result<()> {
        self.oreopen(path, O_WRONLY|O_CREAT|O_TRUNC)
    }
}

impl Reopen for Stderr {
    fn reopen(&mut self, path: &String) -> Result<()> {
        self.oreopen(path, O_WRONLY|O_CREAT|O_TRUNC)
    }
}

pub fn fail<T: Display>(msg: T) -> ! {
    let mut f = Stderr::own().file;
    writeln!(f, "{}", msg).unwrap_or(());
    f.flush().unwrap_or(());
    unsafe {
        exit(EXIT_FAILURE);
    }
}
