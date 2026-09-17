#![no_std]
#![no_main]
use dstd::prelude::*;
use dstd::collections::HashMap;

dstd::main!(main);
fn main() {
    let mut map = HashMap::new();
    map.insert("Key 1", "Value 1");
    map.insert("Key 2", "Value 1");
    assert_eq!(map.insert("Key 2", "Value 2"), Some("Value 1"));
    map.insert("Key 3", "Value 3");
    println!("Contents of the hash map: {map:#?}");
    println!();
    println!("When printed manually:");
    println!("Key 1: {}", map["Key 1"]);
    println!("Key 2: {}", map["Key 2"]);
    println!("Key 3: {}", map["Key 3"]);
}
