use super::*;

#[test]
fn test_find_sum_of_successors() {
    let contents = include_str!("../test.txt");
    let value = sum_of_successors(contents);

    assert_eq!(114, value);
}