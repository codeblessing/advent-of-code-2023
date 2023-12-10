use super::*;

#[test]
fn test_error_margin_product() {
    let contents = include_str!("../test.txt");
    let product = error_margin_product(contents);

    assert_eq!(288, product);
}

#[test]
fn test_corrected_error_margin_product() {
    let contents = include_str!("../test.txt");
    let product = corrected_error_margin_product(contents);

    assert_eq!(71503, product);
}
