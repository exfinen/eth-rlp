use std::fmt;

pub type IndexType = usize;

pub struct ByteStream<'a> {
  index: IndexType,
  buf: &'a [u8],
}

pub enum SerErr {
  NoLengthHeader(IndexType),
  NoData(usize, IndexType),
  NoLengthSize(usize, IndexType),
  RedundantData(IndexType),
}

impl fmt::Debug for SerErr {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      SerErr::NoLengthHeader(idx) => write!(f, format!("No length header at {}", idx)),
      SerErr::NoData(size, idx) => write!(f, format!("No data of size {} at {}", size, idx)),
      SerErr::NoLengthSize(size, idx) => write!(f, format!("No length size of {} at {}", size, idx)),
      SerErr::RedundantData(idx) => write!(f, format!("Redundant data found at {}", idx)),
    }
  }
}

pub enum Result<'a> {
  Bytes(&'a [u8]),
  Fail(IndexType),
}

impl<'a> ByteStream<'a> {
  pub fn new(buf: &'a [u8]) -> Self {
    ByteStream {
      index: 0,
      buf,
    }
  }

  pub fn get_index(&self) -> IndexType {
    self.index
  }

  pub fn is_empty(&self) -> bool {
    self.buf.len() == self.index
  }

  pub fn take(&mut self, n: usize) -> Result {
    if self.index + n > self.buf.len() {
      return Result::Fail(self.index)
    }
    let v = &self.buf[self.index..self.index + n];
    self.index += n;
    Result::Bytes(v)
  }
}