#![no_std]
#![no_main]
use dstd::collections::HashMap;
use dstd::println;

dstd::main!(main);
fn main() {
    let mut map = HashMap::new();

    println!("Insert 100K elements");
    for i in 1..100000 {
        map.insert(i, i);
    }

    println!("Check");
    for i in 1..100000 {
        assert_eq!(map[&i], i);
    }

    println!("Remove odd items");
    for i in 1..100000 {
        if i % 2 == 1 {
            map.remove(&i);
        }
    }

    println!("Check");
    for i in 1..100000 {
        if i % 2 == 0 {
            assert_eq!(map[&i], i);
        } else {
            assert_eq!(map.get(&i), None);
        }
    }

    println!("Remove even items with retain");
    map.retain(|_k, &mut v| v % 2 == 1);

    println!("Final check");
    assert!(map.is_empty());
}
