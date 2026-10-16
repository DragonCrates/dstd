use crate::io::Result;

pub enum SeekFrom {
    Start(u64),
    Current(i64),
    End(i64),
}

impl SeekFrom {
    pub(crate) fn to_flags(&self) -> (i64, i32) {
        match *self {
            // FILE_BEGIN or SEEK_SET
            SeekFrom::Start(off) => (off as i64, 0),
            // FILE_CURRENT or SEEK_CUR
            SeekFrom::Current(off) => (off, 1),
            // FILE_END or SEEK_END
            SeekFrom::End(off) => (off, 2),
        }
    }
}

pub trait Seek {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64>;

    fn rewind(&mut self) -> Result<()> {
        self.seek(SeekFrom::Start(0))?;
        Ok(())
    }

    fn stream_len(&mut self) -> Result<u64> {
        let pos = self.seek(SeekFrom::Current(0))?;
        let len = self.seek(SeekFrom::End(0))?;
        self.seek(SeekFrom::Start(pos))?;
        Ok(len)
    }
 
    fn stream_position(&mut self) -> Result<u64> {
        self.seek(SeekFrom::Current(0))
    }

    fn seek_relative(&mut self, offset: i64) -> Result<()> {
        self.seek(SeekFrom::Current(offset))?;
        Ok(())
    }
}
