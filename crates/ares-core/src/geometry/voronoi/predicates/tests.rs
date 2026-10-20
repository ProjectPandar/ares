// Test-first isolation of the not-yet-written MP_Float binary arithmetic port.
// Failure inventory: signed carry/borrow, exponent alignment, cancellation,
// zero sign, multiplication limb carry, overflow/underflow of a float filter.
use super::{Scalar, exact::Exact, interval::Interval};

#[test]
fn exact_alignment_and_signed_cancellation() {
    let a = Exact::from_double(2.0_f64.powi(100));
    let b = Exact::from_double(1.0);
    assert_eq!(((a.clone() + b.clone()) - a.clone()).sign(), 1);
    assert_eq!(((a.clone() - b.clone()) - a.clone()).sign(), -1);
    assert_eq!((a.clone() - a).sign(), 0);
    assert_eq!(((-b.clone()) + b).sign(), 0);
}

#[test]
fn exact_products_survive_f64_underflow_and_overflow() {
    let tiny = Exact::from_double(f64::from_bits(1));
    assert_eq!((tiny.clone() * tiny.clone()).sign(), 1);
    assert_eq!(((-tiny.clone()) * tiny).sign(), -1);
    let large = Exact::from_double(f64::MAX);
    assert_eq!((large.clone() * large.clone()).sign(), 1);
    let cancellation = (large.clone() * large.clone()) - (large.clone() * large);
    assert_eq!(cancellation.sign(), 0);
}

#[test]
fn interval_filter_never_claims_a_sign_for_cancellation() {
    let a = Interval::from_double(1.0);
    let b = Interval::from_double(1.0);
    assert_eq!((a - b).sign(), None);
    let zero = Interval::from_double(0.0);
    let max = Interval::from_double(f64::MAX);
    assert_eq!(((max * max) * zero).sign(), None);
}
