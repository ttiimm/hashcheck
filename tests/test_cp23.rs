use cyclic_poly_23::CyclicPoly32;
use proptest::prelude::*;

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
        let mut rolling_hash_value = hasher.update(&data[0..window]);
        for i in 1..=(data.len() - window) {
            let leaving = data[i - 1];
            let entering = data[i + window - 1];
            rolling_hash_value = hasher.rotate(leaving, entering);
            let mut tester = CyclicPoly32::new(window);
            let expected = tester.update(&data[i..i+window]);
            assert_eq!(rolling_hash_value, expected, "Hash mismatched at index {}", i);
        }
    }

}
