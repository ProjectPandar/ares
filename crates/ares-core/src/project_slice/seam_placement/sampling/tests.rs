use super::Mt19937_64;

#[test]
fn mt19937_64_matches_reference_sequence() {
    let mut random = Mt19937_64::new(5489);
    assert_eq!(random.next_u64(), 14_514_284_786_278_117_030);
    assert_eq!(random.next_u64(), 4_620_546_740_167_642_908);
    assert_eq!(random.next_u64(), 13_109_570_281_517_897_720);
}
