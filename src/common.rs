use byteorder::{ByteOrder, BigEndian};

pub fn get_in_binary(n: &usize) -> (u8, Vec<u8>) {
  let mut buf = [0u8; 8];
  BigEndian::write_uint(&mut buf, *n as u64, 8);

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

#[test]
fn zero() {
  let n = 0usize;
  let (len, bin) = get_in_binary(&n);
  assert_eq!(len, 0);
  assert_eq!(bin.len(), 0);
}

#[test]
fn one() {
  let n = 1usize;
  let (len, bin) = get_in_binary(&n);
  assert_eq!(len, 1);
  assert_eq!(bin.len(), 1);
  assert_eq!(bin[0], 1);
}

#[test]
fn ff_plus_1() {
  let n = 1usize + 0xff;
  let (len, bin) = get_in_binary(&n);
  assert_eq!(len, 2);
  assert_eq!(bin.len(), 2);
  assert_eq!(bin[0], 1);
  assert_eq!(bin[1], 0);
}

#[test]
fn ffff_ffff_ffff_ffff() {
  let n = 0xffff_ffff_ffff_ffff;
  let (len, bin) = get_in_binary(&n);
  assert_eq!(len, 8);
  assert_eq!(bin.len(), 8);
  for i in 0..8 {
    assert_eq!(bin[i], 0xff);
  }
}


