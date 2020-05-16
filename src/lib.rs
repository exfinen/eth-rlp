mod byte_stream;
mod item;

use byteorder::{ByteOrder, BigEndian};
use crate::item::Item;
use crate::byte_stream::{ByteStream, Result::Bytes, Result::Fail, SerErr};

pub struct Rlp(Item);

impl Rlp {
  fn serialize_list(num_items: usize, st: &mut ByteStream) -> Result<Item, SerErr> {
    let mut items = vec![];
    for _ in 0..num_items {
      let item = match Rlp::decode_byte_stream(st) {
        Ok(x) => x,
        Err(e) => return Err(e),
      };
      items.push(item);
    }
    Ok(Item::List(items))
  }

  fn decode_byte_stream(st: &mut ByteStream) -> Result<Item, SerErr> {
    match st.take(1) {
      Fail(index) => return Err(SerErr::NoLengthHeader(index)),
      Bytes(hdr) => {
        let hdr = hdr[0];
        if hdr <= 0x7f {
          Ok(Item::Str(vec![hdr]))
        } else if hdr <= 0xb7 {
          let len = hdr - 0x80;
          match st.take(len as usize) {
            Fail(index) => return Err(SerErr::NoData(hdr as usize, index)),
            Bytes(xs) => Ok(Item::Str(xs.to_vec())),
          }
        } else if hdr <= 0xbf {
          let len_bytes_len = hdr as usize - 0xb7; // range of len_bytes_len is 1 to 8
          match st.take(len_bytes_len) {
            Fail(index) => return Err(SerErr::NoLengthSize(len_bytes_len, index)),
            Bytes(len_bytes) => {
              let len = BigEndian::read_uint(len_bytes, len_bytes_len);

              match st.take(len as usize) {
                Fail(p) => return Err(SerErr::NoData(len as usize, p)),
                Bytes(xs) => Ok(Item::Str(xs.to_vec())),
              }
            }
          }
        } else if hdr <= 0xf7 {
          let num_items = hdr - 0xc0;  // number of items in the list, not the legth of all items in bytes
          let item = Rlp::serialize_list(num_items as usize, st)?;
          Ok(item)

        } else {
          let len_bytes_len = hdr as usize - 0xf7;
          match st.take(len_bytes_len) {
            Fail(index) => return Err(SerErr::NoLengthSize(len_bytes_len, index)),
            Bytes(len_bytes) => {
              let num_items = BigEndian::read_uint(len_bytes, len_bytes_len);
              let item = Rlp::serialize_list(num_items as usize, st)?;
              Ok(item)
            }
          }
        }
      },
    }
  }

  pub fn decode(&self, byte_array: &[u8]) -> Result<Self, SerErr> {
    let mut st = ByteStream::new(byte_array);
    let item = Rlp::decode_byte_stream(&mut st)?;
    if st.is_empty() {
      Ok(Rlp(item))
    } else {
      Err(SerErr::RedundantData(st.get_index()))
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

  fn encode_item(item: &Item) -> Vec<u8> {
    match item {
      Item::Str(bs) => {
        let len = bs.len();
        if len == 1 && bs[0] <= 0x7f {
          bs.clone()
        } else if len <= 55 {
          let bs2 = vec![0x80 + len as u8];
          bs2.append(&mut bs.clone());
          bs2
        } else {
          let (len_binary_size, mut len_binary) = Rlp::get_in_binary(&len);
          let bs2 = vec![0xb7 + len_binary_size];
          bs2.append(&mut len_binary);
          bs2.append(&mut bs.clone());
          bs2
        }
      },
      Item::List(items) => {
        let (bs, len) = items.into_iter().fold((vec![], 0), |acc, item| {
          let (bs, len) = acc;
          let child_bs = Rlp::encode_item(item);
          bs.append(&mut child_bs);
          (bs, bs.len())
        });
        if len <= 55 {
          let bs2 = vec![0xc0 + len as u8];
          bs2.append(&mut bs);
          bs2
        } else {
          let (len_binary_size, mut len_binary) = Rlp::get_in_binary(&len);
          let bs2 = vec![0xf7 + len_binary_size];
          bs2.append(&mut len_binary);
          bs2.append(&mut bs);
          bs2
        }
      },
    }
  }

  pub fn encode(&self) -> Vec<u8> {
    let mut acc = Vec::<u8>::new();
    Rlp::encode_item(&self.0)
  }
}

#[test]
fn foo_test() {
}