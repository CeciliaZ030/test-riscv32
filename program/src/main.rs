//! A simple program to be proven inside the zkVM.
#![no_main]

// use std::sync::OnceLock;
sp1_zkvm::entrypoint!(main);

// use raiko_lib::{
//     builder::{BlockBuilderStrategy, TaikoStrategy},
//     input::{GuestInput, GuestOutput, WrappedHeader},
//     protocol_instance::{assemble_protocol_instance, EvidenceType},
// };
// use revm_precompile::zk_op::Operation;
// use zk_op::Sp1Operator;

// mod zk_op;

#[cfg(test)]
mod other_test;

pub fn main() {

    // revm_precompile::zk_op::ZKVM_OPERATOR.get_or_init(|| Box::new(Sp1Operator {}));
    // revm_precompile::zk_op::ZKVM_OPERATIONS
    //     .set(Box::new(vec![
    //         Operation::Bn128Add,
    //         Operation::Bn128Mul,
    //         Operation::Secp256k1,
    //     ]))
    //     .expect("Failed to set ZkvmOperations");

    // let build_result = TaikoStrategy::build_from(&GuestInput::default());


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
