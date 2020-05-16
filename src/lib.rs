use byteorder::{ByteOrder, BigEndian};
use std::fmt;

pub enum Item {
  List(usize, Vec<Item>),
  Str(Vec<u8>),
}

pub struct Rlp(Item);

struct ByteStream<'a> {
  p: usize,
  xs: &'a [u8],
}

pub type StreamIndex = usize;

pub enum SerErr {
  NoLengthHeader(StreamIndex),
  NoData(usize, StreamIndex),
  NoLengthSize(usize, StreamIndex),
  RedundantData(StreamIndex),
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

enum TakeResult<'a> {
  Ok(&'a [u8]),
  Err(StreamIndex),
}

impl<'a> ByteStream<'a> {
  fn new(xs: &'a [u8]) -> Self {
    ByteStream {
      p: 0,
      xs,
    }
  }

  fn is_empty(&self) -> bool {
    self.xs.len() == self.p
  }

  fn take(&mut self, n: usize) -> TakeResult {
    if self.p + n > self.xs.len() {
      return TakeResult::Err(self.p)
    }
    let v = &self.xs[self.p..self.p + n];
    self.p += n;
    TakeResult::Ok(v)
  }
}

fn serialize_list(num_items: usize, st: &mut ByteStream) -> Result<(usize, Item), SerErr> {
  let mut xs = vec![];
  let mut total_len = 0;
  for _ in 0..num_items {
    let (len, item) = match Rlp::decode_byte_stream(st) {
      Ok(x) => x,
      Err(e) => return Err(e),
    };
    xs.push(item);
    total_len += len;
  }
  Ok((total_len, Item::List(total_len, xs)))
}

impl Rlp {
  fn decode_byte_stream(st: &mut ByteStream) -> Result<(usize, Item), SerErr> {
    match st.take(1) {
      TakeResult::Err(p) => return Err(SerErr::NoLengthHeader(p)),
      TakeResult::Ok(hdr) => {
        let hdr = hdr[0];
        if hdr <= 0x7f {
          Ok((1, Item::Str(vec![hdr])))
        } else if hdr <= 0xb7 {
          let len = hdr - 0x80;
          match st.take(len as usize) {
            TakeResult::Err(p) => return Err(SerErr::NoData(hdr as usize, p)),
            TakeResult::Ok(xs) => Ok((
              len as usize + 1,
              Item::Str(xs.to_vec())
            )),
          }
        } else if hdr <= 0xbf {
          let len_bytes_len = hdr as usize - 0xb7; // range of len_bytes_len is 1 to 8
          match st.take(len_bytes_len) {
            TakeResult::Err(p) => return Err(SerErr::NoLengthSize(len_bytes_len, p)),
            TakeResult::Ok(len_bytes) => {
              let len = BigEndian::read_uint(len_bytes, len_bytes_len);

              match st.take(len as usize) {
                TakeResult::Err(p) => return Err(SerErr::NoData(len as usize, p)),
                TakeResult::Ok(xs) => Ok((
                  1usize + len_bytes_len,
                  Item::Str(xs.to_vec())
                )),
              }
            }
          }
        } else if hdr <= 0xf7 {
          let num_items = hdr - 0xc0;  // number of items in the list, not the legth of all items in bytes
          let (len, item) = serialize_list(num_items as usize, st)?;
          Ok((1 + len, item))

        } else {
          let len_bytes_len = hdr as usize - 0xf7;
          match st.take(len_bytes_len) {
            TakeResult::Err(p) => return Err(SerErr::NoLengthSize(len_bytes_len, p)),
            TakeResult::Ok(len_bytes) => {
              let num_items = BigEndian::read_uint(len_bytes, len_bytes_len);
              let (len, item) = serialize_list(num_items as usize, st)?;

              Ok((1 + len_bytes_len + len, item))
            }
          }
        }
      },
    }
  }

  pub fn decode(&self, byte_array: &[u8]) -> Result<Self, SerErr> {
    let mut st = ByteStream::new(byte_array);
    let (_, item) = Rlp::decode_byte_stream(&mut st)?;
    if st.is_empty() {
      Ok(Rlp(item))
    } else {
      Err(SerErr::RedundantData(st.p))
    }
  }

  fn get_in_binary(n: &usize) -> (u8, Vec<u8>) {
    let mut buf = [0u8; 8];
    BigEndian::write_uint(&mut buf, *n as u64, 8);  // TODO support item of size > u32

    let mut binary_size = 8;
    while binary_size > 0 {
      let b = buf[8 - binary_size];
      if b > 0 {
        break
      }
      binary_size -= 1;
    }
    (binary_size as u8, buf[8 - binary_size..].to_vec())
  }

  fn encode_item(item: &Item, acc: &mut Vec<u8>) {
    match item {
      Item::Str(bs) => {
        let len = bs.len();
        if len == 1 && bs[0] <= 0x7f {
          acc.push(0x80);
        } else if len <= 55 {
          acc.push(0x80 + len as u8);
          acc.append(&mut bs.clone());
        } else {
          let (len_binary_size, mut len_binary) = Rlp::get_in_binary(&len);
          acc.push(0xb7 + len_binary_size);
          acc.append(&mut len_binary);
          acc.append(&mut bs.clone());
        }
      },
      Item::List(len, items) => {
        if len <= &55 {
          acc.push(0xc0 + items.len() as u8);
        } else {
          let (len_binary_size, mut len_binary) = Rlp::get_in_binary(len);
          acc.push(0xf7 + len_binary_size);
          acc.append(&mut len_binary);
        }
        for item in items {
          Rlp::encode_item(item, acc);
        }
      },
    }
  }

  pub fn encode(&self) -> Vec<u8> {
    let mut acc = Vec::<u8>::new();
    Rlp::encode_item(&self.0, &mut acc);
    acc
  }
}

#[test]
fn foo_test() {
}