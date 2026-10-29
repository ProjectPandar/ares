//! Exact signed binary floats for the MP_Float fallback: integer * 2^exponent.
use super::Scalar;
use std::{
    cmp::Ordering,
    ops::{Add, Mul, Neg, Sub},
};

#[derive(Clone)]
pub(super) struct Exact {
    sign: i8,
    digits: Vec<u32>,
    exponent: i32,
}

impl Exact {
    fn normalized(sign: i8, mut digits: Vec<u32>, exponent: i32) -> Self {
        while digits.last() == Some(&0) {
            digits.pop();
        }
        Self {
            sign: if digits.is_empty() { 0 } else { sign },
            digits,
            exponent,
        }
    }
    pub(super) fn sign(&self) -> i8 {
        self.sign
    }
    fn shifted(&self, exponent: i32) -> Vec<u32> {
        let shift = (self.exponent - exponent) as usize;
        let mut out = vec![0; shift / 32];
        let mut carry = 0_u64;
        for &digit in &self.digits {
            let value = (u64::from(digit) << (shift % 32)) | carry;
            out.push(value as u32);
            carry = value >> 32;
        }
        if carry != 0 {
            out.push(carry as u32);
        }
        out
    }
}
impl Scalar for Exact {
    fn from_double(value: f64) -> Self {
        let bits = value.to_bits();
        let exponent = ((bits >> 52) & 2047) as i32;
        let mantissa = (bits & ((1_u64 << 52) - 1)) | if exponent == 0 { 0 } else { 1_u64 << 52 };
        Self::normalized(
            if bits >> 63 == 0 { 1 } else { -1 },
            vec![mantissa as u32, (mantissa >> 32) as u32],
            if exponent == 0 {
                -1074
            } else {
                exponent - 1023 - 52
            },
        )
    }
}
impl Neg for Exact {
    type Output = Self;
    fn neg(mut self) -> Self {
        self.sign = -self.sign;
        self
    }
}
impl Add for Exact {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        if self.sign == 0 {
            return rhs;
        }
        if rhs.sign == 0 {
            return self;
        }
        let exponent = self.exponent.min(rhs.exponent);
        let mut a = self.shifted(exponent);
        let mut b = rhs.shifted(exponent);
        if self.sign == rhs.sign {
            a.resize(a.len().max(b.len()), 0);
            let mut carry = 0_u64;
            for (i, digit) in a.iter_mut().enumerate() {
                let sum = u64::from(*digit) + u64::from(b.get(i).copied().unwrap_or(0)) + carry;
                *digit = sum as u32;
                carry = sum >> 32;
            }
            if carry != 0 {
                a.push(carry as u32);
            }
            return Self::normalized(self.sign, a, exponent);
        }
        let ordering = a
            .len()
            .cmp(&b.len())
            .then_with(|| a.iter().rev().cmp(b.iter().rev()));
        let sign = if ordering == Ordering::Less {
            std::mem::swap(&mut a, &mut b);
            rhs.sign
        } else {
            self.sign
        };
        let mut borrow = 0_i64;
        for (i, digit) in a.iter_mut().enumerate() {
            let value = i64::from(*digit) - i64::from(b.get(i).copied().unwrap_or(0)) - borrow;
            *digit = value as u32;
            borrow = i64::from(value < 0);
        }
        Self::normalized(sign, a, exponent)
    }
}
impl Sub for Exact {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        self + (-rhs)
    }
}
impl Mul for Exact {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        let mut out = vec![0_u32; self.digits.len() + rhs.digits.len()];
        for (i, &a) in self.digits.iter().enumerate() {
            let mut carry = 0_u64;
            for (j, &b) in rhs.digits.iter().enumerate() {
                let value = u64::from(a) * u64::from(b) + u64::from(out[i + j]) + carry;
                out[i + j] = value as u32;
                carry = value >> 32;
            }
            out[i + rhs.digits.len()] = carry as u32;
        }
        Self::normalized(self.sign * rhs.sign, out, self.exponent + rhs.exponent)
    }
}
