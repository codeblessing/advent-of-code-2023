use super::*;

#[test]
fn test_total_cards_score() {
    let contents = include_str!("../test.txt");

    let total_score = contents.lines().map(winning_numbers_count).sum::<usize>();

    assert_eq!(13, total_score);
}

#[test]
fn test_scratchcards_count() {
    let contents = include_str!("../test.txt");

    let cards_count = count_total_cards(contents);

    assert_eq!(30, cards_count);
}
