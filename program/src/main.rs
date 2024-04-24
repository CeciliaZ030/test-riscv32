//! A simple program to be proven inside the zkVM.
#![no_main]
sp1_zkvm::entrypoint!(main);

use raiko_lib::{
    builder::{BlockBuilderStrategy, TaikoStrategy},
    input::{GuestInput, GuestOutput, WrappedHeader},
    protocol_instance::{assemble_protocol_instance, EvidenceType},
};
use revm_precompile::zk_op::ZkOperation;
use zk_op::Sp1Operator;

mod zk_op;

pub fn main() {
    // let input = sp1_zkvm::io::read::<GuestInput>();

    revm_precompile::zk_op::ZKVM_OPERATOR.get_or_init(|| Box::new(Sp1Operator {}));
    revm_precompile::zk_op::ZKVM_OPERATIONS
        .set(Box::new(vec![
            ZkOperation::Bn128Add,
            ZkOperation::Bn128Mul,
            ZkOperation::Secp256k1,
        ]))
        .expect("Failed to set ZkvmOperations");

    // let build_result = TaikoStrategy::build_from(&input);

    // let output = match &build_result {
    //     Ok((header, _mpt_node)) => {
    //         let pi = assemble_protocol_instance(&input, header)
    //             .expect("Failed to assemble protocol instance")
    //             .instance_hash(EvidenceType::Succinct);
    //         GuestOutput::Success((
    //             WrappedHeader {
    //                 header: header.clone(),
    //             },
    //             pi,
    //         ))
    //     }
    //     Err(_) => GuestOutput::Failure,
    // };

    // sp1_zkvm::io::commit(&output);
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
        let n = 12;
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
