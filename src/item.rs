#[derive(Debug)]
pub enum Item {
  List(Vec<Item>),
  Str(Vec<u8>),
}