use super::*;

#[test]
fn test_total_cards_score() {
    let contents = include_str!("../test.txt");

    let total_score = contents.lines().map(card_point_score).sum::<i32>();

    assert_eq!(13, total_score);
}