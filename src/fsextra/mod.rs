use libc::funcs::posix88::fcntl::open;
use libc::funcs::posix88::unistd::dup2;
use libc::consts::os::posix88::{ O_RDONLY, O_WRONLY, O_CREAT };
use std::io::{ Error, Result };
use std::os::unix::io::{ AsRawFd, RawFd, FromRawFd };
use std::os::unix::fs::PermissionsExt;
use std::os::unix::fs::MetadataExt;
use std::fs::{ File, Metadata };
use std::ffi::CString;

pub struct Stdin  { pub file: File }
pub struct Stdout { pub file: File }
pub struct Stderr { pub file: File }

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
    pub fn own() -> Stdin  { unsafe { Stdin  { file: File::from_raw_fd(0) } } }
}
impl Stdout {
    pub fn own() -> Stdout { unsafe { Stdout { file: File::from_raw_fd(1) } } }
}
impl Stderr {
    pub fn own() -> Stderr { unsafe { Stderr { file: File::from_raw_fd(2) } } }
}

impl FileInfo for File {
    fn metadata(&self) -> Result<Metadata> { self.metadata() }
}

trait ReopenMode {
    fn oreopen(&mut self, path: &String, mode: i32) -> Result<()>;
}

impl<T> ReopenMode for T where T : AsRawFd + FileInfo {
    fn oreopen(&mut self, path: &String, mode: i32) -> Result<()> {
        let fd = self.as_raw_fd();
        let cpath = CString::new(&path[..]).unwrap();
        let metadata = try!(self.metadata());
        let file = unsafe { open(cpath.as_ptr(), mode, metadata.mode()) };
        if file == -1 {
            return Err(Error::last_os_error());
        }
        if unsafe { dup2(file, fd) == -1 } {
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
        self.oreopen(path, O_WRONLY|O_CREAT)
    }
}

impl Reopen for Stderr {
    fn reopen(&mut self, path: &String) -> Result<()> {
        self.oreopen(path, O_WRONLY|O_CREAT)
    }
}