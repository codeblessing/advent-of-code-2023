use super::*;

#[test]
fn test_total_winnings() {
    let contents = include_str!("../test.txt");
    let winnings = total_winnings(contents);

    assert_eq!(6440, winnings);
}
