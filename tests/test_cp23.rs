use cyclic_poly_23::CyclicPoly32;

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
