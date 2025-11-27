//! Unit tests for common math utilities

use perptrix::common::math::*;

#[test]
fn test_sma() {
    let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    assert_eq!(sma(&values, 3), Some(4.0));
    assert_eq!(sma(&values, 5), Some(3.0));
    assert_eq!(sma(&values, 10), None);
}

#[test]
fn test_ema() {
    let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let result = ema(&values, 3);
    assert!(result.is_some());
    assert!(result.unwrap() > 0.0);
}

#[test]
fn test_standard_deviation() {
    let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let result = standard_deviation(&values, 5);
    assert!(result.is_some());
    assert!(result.unwrap() > 0.0);
}

#[test]
fn test_true_range() {
    let tr = true_range(10.0, 8.0, 9.0);
    assert_eq!(tr, 2.0);
    
    let tr2 = true_range(10.0, 8.0, 7.0);
    assert_eq!(tr2, 3.0);
}

