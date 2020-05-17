#[derive(Debug)]
pub enum Item {
  List(Vec<Item>),
  Str(Vec<u8>),
}

impl From<&str> for Item {
  fn from(s: &str) -> Self {
    Item::Str(s.as_bytes().to_vec())
  }
}

impl From<usize> for Item {
  fn from(n: usize) -> Self {
    use crate::common::get_in_binary;

    let (_, bin_n) = get_in_binary(&n);
    Item::Str(bin_n)
  }
}