#[derive(Debug)]
pub enum Item {
  List(Vec<Item>),
  Str(Vec<u8>),
}

impl Item {
  pub fn empty_str() -> Self {
    Item::Str(vec![])
  }
}

impl From<&str> for Item {
  fn from(s: &str) -> Self {
    Item::Str(s.as_bytes().to_vec())
  }
}