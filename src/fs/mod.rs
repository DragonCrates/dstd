use crate::io::{Result, Read, Write, Seek, SeekFrom};
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

pub type Handle = sys::Handle;

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

    pub fn open<'a, P: Into<Path<'a>>>(&self, name: P) -> Result<File> {
        let name = name.into();
        let mut buf = [0; 256];
        let name_os = name.to_os_with(&mut buf)?;

        let handle = sys::open(&name_os, self)?;
        Ok(File { handle })
    }
}

pub struct File {
    handle: Handle,
}

impl File {
    pub fn create<'a, P: Into<Path<'a>>>(name: P) -> Result<File> {
        File::options().write(true).create(true).truncate(true).open(name)
    }

    pub fn create_new<'a, P: Into<Path<'a>>>(name: P) -> Result<File> {
        File::options().write(true).create_new(true).open(name)
    }

    pub fn open<'a, P: Into<Path<'a>>>(name: P) -> Result<File> {
        File::options().read(true).open(name)
    }

    pub fn options() -> OpenOptions {
        OpenOptions::new()
    }
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        sys::read(self.handle, buf)
    }
}

impl Write for File {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        sys::write(self.handle, buf)
    }
}

impl Seek for File {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        sys::seek(self.handle, pos)
    }
}
