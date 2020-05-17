use std::fmt;

pub type IndexType = usize;

pub struct ByteStream<'a> {
  index: IndexType,
  buf: &'a [u8],
}

pub enum SerErr {
  NoChildTree(usize, IndexType),
  NoLengthHeader(IndexType),
  NoData(usize, IndexType),
  NoLengthSize(usize, IndexType),
  RedundantData(IndexType),
  BadSingleByteEncoding(u8, IndexType),
}

impl fmt::Debug for SerErr {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      SerErr::NoChildTree(len, idx) => write!(f, "No child tree of length {} at {}", len, idx),
      SerErr::NoLengthHeader(idx) => write!(f, "No length header at {}", idx),
      SerErr::NoData(len, idx) => write!(f, "No data of size {} at {}", len, idx),
      SerErr::NoLengthSize(len, idx) => write!(f, "No length size of {} at {}", len, idx),
      SerErr::RedundantData(idx) => write!(f, "Redundant data found at {}", idx),
      SerErr::BadSingleByteEncoding(x, idx) => write!(f, "{} not encoded as single byte at {}", x, idx),
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