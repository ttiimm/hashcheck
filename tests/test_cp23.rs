use cyclic_poly_23::CyclicPoly32;
use proptest::prelude::*;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{LazyLock, Mutex};
use std::thread::sleep;
use std::time;

// Static HashMap to check for collisions across prop tests
static COLLISION_MAP: LazyLock<Mutex<HashMap<u32, Vec<Vec<u8>>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static COUNTER: AtomicUsize = AtomicUsize::new(0);
static TO_RUN: u32 = 250_000;

#[test]
fn test_basic_32() {
    let data = [0, 1, 2, 3];
    let mut hasher = CyclicPoly32::new(4);
    let hash_value = hasher.update(&data);
    assert_eq!(hash_value, 0xecf6e475);
}

#[test]
fn test_basic_32_roll() {
    let data = [8, 0, 1, 2, 3];
    let mut hasher = CyclicPoly32::new(4);
    let previous = hasher.update(&data[0..4]);
    // rotate out the value in 0 and put in the value at 4th
    let next = hasher.rotate(data[0], data[4]);
    assert_ne!(previous, next);
    assert_eq!(next, 0xecf6e475);
}

// a vector of random u8 of size between 1-1000 and a window between 1-1000
fn rolling_hash_strategy() -> impl Strategy<Value = (Vec<u8>, usize)> {
    (
        proptest::collection::vec(any::<u8>(), 1..1000),
        1usize..1000usize,
    )
        .prop_filter("Window must be less than the data", |(data, window)| {
            window <= &data.len()
        })
}

proptest! {
    #[test]
    fn test_hashes_match_after_rolling((data, window) in rolling_hash_strategy()) {
        let mut hasher = CyclicPoly32::new(window);
        let _ = hasher.update(&data[0..window]);
        for i in 1..=(data.len() - window) {
            let leaving = data[i - 1];
            let entering = data[i + window - 1];
            let rolling_hash_value = hasher.rotate(leaving, entering);
            let mut tester = CyclicPoly32::new(window);
            let expected = tester.update(&data[i..i+window]);
            proptest::prop_assert_eq!(rolling_hash_value, expected, "Hash mismatched at index {}", i);
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(TO_RUN))]

    #[test]
    fn test_hash_collisions(data in proptest::collection::vec(any::<u8>(), 1..20)) {
        let mut hasher = CyclicPoly32::new(data.len());
        let value = hasher.update(&data);
        let mut map = COLLISION_MAP.lock().unwrap();

        if let Some(existing_data_list) = map.get_mut(&value) {
            existing_data_list.push(data);
        } else {
            map.insert(value, vec![data]);
        }
        COUNTER.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn summarize_collisions() {
    // wait for the collision proptest to finish
    let mut count = COUNTER.load(Ordering::SeqCst);
    while count < TO_RUN.try_into().unwrap() {
        count = COUNTER.load(Ordering::SeqCst);
        sleep(time::Duration::from_millis(10));
    }

    let map = COLLISION_MAP.lock().unwrap();
    let mut total_collisions = 0;
    let mut collision_groups = 0;

    for (hash, data_list) in map.iter() {
        if data_list.len() > 1 {
            let mut unique_data = std::collections::HashSet::new();
            for data in data_list {
                unique_data.insert(data);
            }

            if unique_data.len() > 1 {
                collision_groups += 1;
                total_collisions += unique_data.len();
                println!(
                    "Collision group for hash 0x{:x}: {} unique inputs",
                    hash,
                    unique_data.len()
                );
                for data in unique_data.iter() {
                    println!("  {:?}", data);
                }
            }
        }
    }

    println!("Total collision summary:");
    println!("  Collision groups: {}", collision_groups);
    println!("  Total unique inputs in collisions: {}", total_collisions);
    println!("  Total hashes tested: {}", map.len());
}
