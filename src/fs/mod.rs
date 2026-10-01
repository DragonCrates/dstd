use crate::io::{self, Read, Write, Seek, SeekFrom};
use crate::path::Path;

#[cfg(windows)]
crate::block! {
    mod windows;
    use windows as sys;
}

#[cfg(unix)]
crate::block! {
    mod unix;
    use unix as sys;
}

pub type RawHandle = sys::RawHandle;

#[derive(Default)]
pub struct OpenOptions {
    read: bool,
    write: bool,
    append: bool,
    truncate: bool,
    create: bool,
    excl: bool,
}

impl OpenOptions {
    pub fn new() -> OpenOptions {
        OpenOptions::default()
    }

    pub fn read(&mut self, val: bool) -> &mut OpenOptions {
        self.read = val;
        self
    }

    pub fn write(&mut self, val: bool) -> &mut OpenOptions {
        self.write = val;
        self
    }

    pub fn append(&mut self, val: bool) -> &mut OpenOptions {
        self.append = val;
        self
    }

    pub fn truncate(&mut self, val: bool) -> &mut OpenOptions {
        self.truncate = val;
        self
    }

    pub fn create(&mut self, val: bool) -> &mut OpenOptions {
        self.create = val;
        self
    }

    pub fn create_new(&mut self, val: bool) -> &mut OpenOptions {
        if val { self.create = true; }
        self.excl = val;
        self
    }

    pub fn open<'a, P: Into<Path<'a>>>(&self, name: P) -> io::Result<File> {
        let name = name.into();
        let mut buf = [0; 256];
        let name_os = name.to_os_with(&mut buf)?;

        let file = sys::File::open(&name_os, self)?;
        Ok(File { file })
    }
}

pub struct File {
    file: sys::File,
}

impl File {
    pub fn options() -> OpenOptions {
        OpenOptions::new()
    }

    pub fn create<'a, P: Into<Path<'a>>>(name: P) -> io::Result<File> {
        File::options().write(true).create(true).truncate(true).open(name)
    }

    pub fn create_new<'a, P: Into<Path<'a>>>(name: P) -> io::Result<File> {
        File::options().write(true).create_new(true).open(name)
    }

    pub fn open<'a, P: Into<Path<'a>>>(name: P) -> io::Result<File> {
        File::options().read(true).open(name)
    }
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.file.read(buf)
    }
}

impl Write for File {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.file.write(buf)
    }
}

impl Seek for File {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.file.seek(pos)
    }
}
