# eth-rlp

Implementaion of Ethereum RLP encoding/decoding in Rust basd on information available in https://github.com/ethereum/wiki/wiki/RLP

## Usage
Construct an item using below enum:

```rust
pub enum Item {
  List(Vec<Item>),
  Str(Vec<u8>),
}
```

Encode an item with `Rlp::encode` e.g.

```rust
let item = Item::from("dog");
let ba = Rlp::encode(item).unwrap();
assert_eq!(ba, [0x83, 'd' as u8, 'o' as u8, 'g' as u8]);
```

Decode an encoded item (byte array) with `Rlp::decode` e.g.

```rust
// not tested. refer to tests for actual usages.
let ba = [0x83, 'd' as u8, 'o' as u8, 'g' as u8];
match Rlp::decode(&ba).unwrap() {
  Item::Str(ba2) => assert_eq!(ba2, "dog".as_bytes().to_vec()),
  ...
}
```

More detailed usage is available in tests in lib.rs

## Helper functions
- To encode empty string, use `Item::empty_str()`
- To encode integer, use `Item::from(usize)` which drops preceeding zeros in byte representation.
