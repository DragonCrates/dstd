use crate::collections::HashMap;

/// Test that [`HashMap`] works as expected
#[test]
fn hashmap_works() {
    let mut map = HashMap::new();

    // Insert 100K elements
    for i in 1..100000 {
        map.insert(i, i);
    }

    // Check
    for i in 1..100000 {
        assert_eq!(map[&i], i);
    }

    // Remove odd items
    for i in 1..100000 {
        if i % 2 == 1 {
            map.remove(&i);
        }
    }

    // Check
    for i in 1..100000 {
        if i % 2 == 0 {
            assert_eq!(map[&i], i);
        } else {
            assert_eq!(map.get(&i), None);
        }
    }
}
