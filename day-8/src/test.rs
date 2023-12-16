use super::*;

#[test]
fn test_step_count() {
    let contents = include_str!("../test.txt");
    let steps = step_count(contents);

    assert_eq!(steps, 2);

    let contents = include_str!("../test_2.txt");
    let steps = step_count(contents);

    assert_eq!(steps, 6);
}

#[test]
fn test_simultaneous_step_count() {
    let contents = include_str!("../test.txt");
    let steps = simultaneous_step_count(contents);

    assert_eq!(steps, 2);

    let contents = include_str!("../test_3.txt");
    let steps = simultaneous_step_count(contents);

    assert_eq!(steps, 6);
}