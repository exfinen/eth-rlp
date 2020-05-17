mod byte_stream;
mod item;
mod common;

use byteorder::{ByteOrder, BigEndian};
use crate::item::Item;
use crate::byte_stream::{ByteStream, Result::Bytes, Result::Fail, SerErr};
use common::get_in_binary;

pub struct Rlp;

impl Rlp {
  fn serialize_list(len: usize, st: &mut ByteStream) -> Result<Item, SerErr> {
    match st.take(len) {
      Fail(index) => return Err(SerErr::NoChildTree(len, index)),
      Bytes(child_buf) => {
        let mut child_st = ByteStream::new(child_buf);
        let mut items = vec![];
        while !child_st.is_empty() {
          let item = Rlp::decode_byte_stream(&mut child_st)?;
          items.push(item);
        }
        Ok(Item::List(items))
      },
    }
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
            Bytes(xs) => {
              if len == 1 && xs[0] <= 0x7f {
                return Err(SerErr::BadSingleByteEncoding(xs[0], st.get_index())) // TODO write test
              }
              Ok(Item::Str(xs.to_vec()))
            },
          }
        } else if hdr <= 0xbf {
          let len_bytes_len = hdr as usize - 0xb7; // range of len_bytes_len is 1 to 8
          if len_bytes_len > 8 {
            return Err(SerErr::LengthTooLarge(len_bytes_len as u8, st.get_index())); // TODO write test
          }
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
          let len = hdr - 0xc0;  // length of the list in bytes
          let item = Rlp::serialize_list(len as usize, st)?;
          Ok(item)

        } else {
          let len_bytes_len = hdr as usize - 0xf7;
          if len_bytes_len > 8 {
            return Err(SerErr::LengthTooLarge(len_bytes_len as u8, st.get_index())); // TODO write test
          }
          match st.take(len_bytes_len) {
            Fail(index) => return Err(SerErr::NoLengthSize(len_bytes_len, index)),
            Bytes(len_bytes) => {
              let len = BigEndian::read_uint(len_bytes, len_bytes_len);
              let item = Rlp::serialize_list(len as usize, st)?;
              Ok(item)
            }
          }
        }
      },
    }
  }

  pub fn decode(byte_array: &[u8]) -> Result<Item, SerErr> {
    let mut st = ByteStream::new(byte_array);
    let item = Rlp::decode_byte_stream(&mut st)?;
    if st.is_empty() {
      Ok(item)
    } else {
      Err(SerErr::RedundantData(st.get_index()))
    }
  }

  fn encode_item(item: &Item) -> Result<Vec<u8>, SerErr> {
    match item {
      Item::Str(bs) => {
        let len = bs.len();
        if len == 1 && bs[0] <= 0x7f {
          Ok(bs.clone())
        } else if len <= 55 {
          let mut bs2 = vec![0x80 + len as u8];
          bs2.append(&mut bs.clone());
          Ok(bs2)
        } else {
          if len > usize::MAX {
            return Err(SerErr::LengthTooLarge(0, 0)); // TODO give params somehow
          }
          let (len_binary_size, mut len_binary) = get_in_binary(&len);
          let mut bs2 = vec![0xb7 + len_binary_size];
          bs2.append(&mut len_binary);
          bs2.append(&mut bs.clone());
          Ok(bs2)
        }
      },
      Item::List(items) => {
        let mut bs = vec![];
        let mut len = 0;
        for item in items {
          let mut child_bs = Rlp::encode_item(item)?;
          let child_len = child_bs.len();
          bs.append(&mut child_bs);
          len += child_len;
        }
        if len <= 55 {
          let mut bs2 = vec![0xc0 + len as u8];
          bs2.append(&mut bs);
          Ok(bs2)
        } else {
          let (len_binary_size, mut len_binary) = get_in_binary(&len);
          let mut bs2 = vec![0xf7 + len_binary_size];
          bs2.append(&mut len_binary);
          bs2.append(&mut bs);
          Ok(bs2)
        }
      },
    }
  }

  pub fn encode(item: Item) -> Result<Vec<u8>, SerErr> {
    Rlp::encode_item(&item)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::Item::{Str, List};

  // The string "dog" = [ 0x83, 'd', 'o', 'g' ]
  #[test]
  fn dog() {
    let in_item = Item::from("dog");
    let bs = Rlp::encode(in_item).unwrap();
    println!(r#"encoded "dog" -> {}"#, hex::encode(&bs));
    assert_eq!(bs, [0x83, 'd' as u8, 'o' as u8, 'g' as u8]);

    match Rlp::decode(&bs).unwrap() {
      List(_) => assert!(false),
      Str(bs2) => {
        println!("decoded {} -> {:?}", hex::encode(&bs), String::from_utf8(bs2.clone()).unwrap());
        assert_eq!(bs2, "dog".as_bytes());
      },
    };
  }

  // The list [ "cat", "dog" ] = [ 0xc8, 0x83, 'c', 'a', 't', 0x83, 'd', 'o', 'g' ]
  #[test]
  fn cat_dog_list() {
    let in_item = Item::List(
      vec![
        Item::from("cat"),
        Item::from("dog"),
      ]
    );
    let bs = Rlp::encode(in_item).unwrap();
    println!(r#"encoded ["cat", "dog"] -> {}"#, hex::encode(&bs));
    assert_eq!(bs, [0xc8, 0x83, 'c' as u8, 'a' as u8, 't' as u8, 0x83, 'd' as u8, 'o' as u8, 'g' as u8]);

    println!("decoding {}", hex::encode(&bs));
    match Rlp::decode(&bs).unwrap() {
      List(xs) => {
        println!("decoded {:?} -> {:?}", hex::encode(&bs), xs);
        assert_eq!(xs.len(), 2);
        if let Str(bs) = &xs[0] { assert_eq!(bs, &"cat".as_bytes().to_vec()) } else { assert!(false) }
        if let Str(bs) = &xs[1] { assert_eq!(bs, &"dog".as_bytes().to_vec()) } else { assert!(false) }
      },
      _ => assert!(false),
    };
  }

  // The empty string ('null') = [ 0x80 ]
  #[test]
  fn empty_str() {
    let in_item = Item::Str(vec![]);
    let bs = Rlp::encode(in_item).unwrap();
    println!(r#"encoded "" -> {}"#, hex::encode(&bs));
    assert_eq!(bs, [0x80]);

    match Rlp::decode(&bs).unwrap() {
      List(_) => assert!(false),
      Str(bs2) => {
        println!("decoded {} -> {:?}", hex::encode(&bs), String::from_utf8(bs2.clone()).unwrap());
        assert_eq!(bs2, "".as_bytes());
      },
    };
  }

  // The empty list = [ 0xc0 ]
  #[test]
  fn empty_list() {
    let in_item = Item::List(vec![]);
    let bs = Rlp::encode(in_item).unwrap();
    println!(r#"encoded [] -> {}"#, hex::encode(&bs));
    assert_eq!(bs, [0xc0]);

    match Rlp::decode(&bs).unwrap() {
      Str(_) => assert!(false),
      List(bs2) => {
        println!("decoded {} -> {:?}", hex::encode(&bs), bs2);
        assert_eq!(bs2.len(), 0);
      },
    };
  }

  // The integer 0 = [ 0x80 ]
  #[test]
  fn integer_0() {
    let in_item = Item::from(0);
    let bs = Rlp::encode(in_item).unwrap();
    println!(r#"encoded int 0 -> {}"#, hex::encode(&bs));
    assert_eq!(bs, [0x80]);

    match Rlp::decode(&bs).unwrap() {
      List(_) => assert!(false),
      Str(bs2) => {
        println!("decoded {} -> {:?}", hex::encode(&bs), bs2);
        assert_eq!(bs2.len(), 0);
      },
    };
  }

  // The encoded integer 0 ('\x00') = [ 0x00 ]
  #[test]
  fn encoded_integer_0() {
    let in_item = Item::Str(vec![0]);
    let bs = Rlp::encode(in_item).unwrap();
    println!(r#"encoded 00 -> {}"#, hex::encode(&bs));
    assert_eq!(bs, [0]);

    match Rlp::decode(&bs).unwrap() {
      List(_) => assert!(false),
      Str(bs2) => {
        println!("decoded {} -> {:?}", hex::encode(&bs), bs2);
        assert_eq!(bs2.len(), 1);
        assert_eq!(bs2[0], 0);
      },
    };
  }

  // The encoded integer 15 ('\x0f') = [ 0x0f ]
  #[test]
  fn encoded_integer_15() {
    let in_item = Item::Str(vec![0x0f]);
    let bs = Rlp::encode(in_item).unwrap();
    println!(r#"encoded 0x0f -> {}"#, hex::encode(&bs));
    assert_eq!(bs, [0x0f]);

    match Rlp::decode(&bs).unwrap() {
      List(_) => assert!(false),
      Str(bs2) => {
        println!("decoded {} -> {:?}", hex::encode(&bs), bs2);
        assert_eq!(bs2.len(), 1);
        assert_eq!(bs2[0], 0x0f);
      },
    };
  }

  // The encoded integer 1024 ('\x04\x00') = [ 0x82, 0x04, 0x00 ]
  #[test]
  fn encoded_integer_1024() {
    let in_item = Item::Str(vec![0x04, 0x00]);
    let bs = Rlp::encode(in_item).unwrap();
    println!(r#"encoded 0x0400 -> {}"#, hex::encode(&bs));
    assert_eq!(bs, [0x82, 0x04, 0x00]);

    match Rlp::decode(&bs).unwrap() {
      List(_) => assert!(false),
      Str(bs2) => {
        println!("decoded {} -> {:?}", hex::encode(&bs), bs2);
        assert_eq!(bs2.len(), 2);
        assert_eq!(bs2[0], 0x04);
        assert_eq!(bs2[1], 0x00);
      },
    };
  }

  // The set theoretical representation of three,
  // [ [], [[]], [ [], [[]] ] ] = [ 0xc7, 0xc0, 0xc1, 0xc0, 0xc3, 0xc0, 0xc1, 0xc0 ]
  #[test]
  fn theoretical_rep_of_3() {
    let in_item = List(vec![
      List(vec![]),
      List(vec![
        List(vec![]),
      ]),
      List(vec![
        List(vec![]),
        List(vec![
          List(vec![]),
        ]),
      ]),
    ]);
    let bs = Rlp::encode(in_item).unwrap();
    println!(r#"encoded [ [], [[]], [ [], [[]] ] ] -> {}"#, hex::encode(&bs));
    assert_eq!(bs, [0xc7, 0xc0, 0xc1, 0xc0, 0xc3, 0xc0, 0xc1, 0xc0]);

    match Rlp::decode(&bs).unwrap() {
      Str(_) => assert!(false),
      List(bs2) => {
        println!("decoded {} -> {:?}", hex::encode(&bs), bs2);
        assert_eq!(bs2.len(), 3);
        // first list []
        match &bs2[0] {
          Str(_) => assert!(false),
          List(bs3) => {
            assert_eq!(bs3.len(), 0);
          },
        }
        // second list [[]]
        match &bs2[1] {
          Str(_) => assert!(false),
          List(bs3) => {
            assert_eq!(bs3.len(), 1);
            match &bs3[0] {
              Str(_) => assert!(false),
              List(bs4) => {
                assert_eq!(bs4.len(), 0);
              },
            }
          },
        }
        // third list [ [], [[]] ]
        match &bs2[2] {
          Str(_) => assert!(false),
          List(bs3) => {
            assert_eq!(bs3.len(), 2);
            match &bs3[0] {
              Str(_) => assert!(false),
              List(bs4) => {
                assert_eq!(bs4.len(), 0);
              },
            }
            match &bs3[1] {
              Str(_) => assert!(false),
              List(bs4) => {
                assert_eq!(bs4.len(), 1);
                match &bs4[0] {
                  Str(_) => assert!(false),
                  List(bs5) => {
                    assert_eq!(bs5.len(), 0);
                  },
                }
              },
            }
          },
        }
      },
    };
  }

  // The string
  //   "Lorem ipsum dolor sit amet, consectetur adipisicing elit" =
  //   [ 0xb8, 0x38, 'L', 'o', 'r', 'e', 'm', ' ', ... , 'e', 'l', 'i', 't' ]
  #[test]
  fn long_str() {
    let s = "Lorem ipsum dolor sit amet, consectetur adipisicing elit";
    let in_item = Item::from(s);
    let bs = Rlp::encode(in_item).unwrap();
    println!(r#"encoded {} -> {}"#, s, hex::encode(&bs));

    let mut exp = vec![0xb8, 0x38];
    exp.append(&mut s.as_bytes().to_vec());
    assert_eq!(bs, exp);

    match Rlp::decode(&bs).unwrap() {
      List(_) => assert!(false),
      Str(bs2) => {
        println!("decoded {} -> {:?}", hex::encode(&bs), bs2);
        assert_eq!(bs2, s.as_bytes().to_vec());
      },
    };
  }
}