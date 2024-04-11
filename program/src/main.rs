//! A simple program to be proven inside the zkVM.
#![no_main]

use std::sync::OnceLock;
sp1_zkvm::entrypoint!(main);

pub trait ZkvmOperator: Send + Sync + 'static {}
pub static ZKVM_OPERATOR: OnceLock<Box<dyn ZkvmOperator>> = OnceLock::new();

pub fn main() {
    // NOTE: values of n larger than 186 will overflow the u128 type,
    // resulting in output that doesn't match fibonacci sequence.
    // However, the resulting proof will still be valid!
    let n = sp1_zkvm::io::read::<u32>() as u128;
    let mut a: u128 = 0;
    let mut b: u128 = 1;
    let mut sum: u128;
    for _ in 1..n {
        sum = a + b;
        a = b;
        b = sum;
    }

    sp1_zkvm::io::commit(&a);
    sp1_zkvm::io::commit(&b);
}

// fn main() {}

#[cfg(test)]
mod ttt {
    use super::{SimpleAlloc, HEAP};

    #[test]
    fn test_fibo() {
        let n = 10;
        let mut a: u128 = 0;
        let mut b: u128 = 1;
        let mut sum: u128;
        for _ in 1..n {
            sum = a + b;
            a = b;
            b = sum;
        }
    }
    
    #[test]
    fn test_fibo2() {
        let n = 10;
        let mut a: u128 = 0;
        let mut b: u128 = 1;
        let mut sum: u128;
        for _ in 1..n {
            sum = a + b;
            a = b;
            b = sum;
        }
    }
}
