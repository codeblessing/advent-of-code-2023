use super::*;

#[test]
fn test_total_winnings() {
    let contents = include_str!("../test.txt");
    let winnings = total_winnings(contents);

    assert_eq!(6440, winnings);
}

#[test]
fn test_modified_total_winnings() {
    let contents = include_str!("../test.txt");
    let winnings = modified_total_winnings(contents);

    assert_eq!(5905, winnings);
}