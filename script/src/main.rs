//! A simple script to generate and verify the proof of a given program.
use sp1_sdk::{SP1Prover, SP1Stdin, SP1Verifier};

// const ELF: &[u8] = include_bytes!("../../program/elf/riscv32im-succinct-zkvm-elf");
// const TEST_ELF: &[u8] = include_bytes!("../../program/elf/riscv32im-succinct-zkvm-elf-test");

fn main() {
    // Generate proof.
    // let mut stdin = SP1Stdin::new();
    // let n = 186u32;
    // stdin.write(&n);
    // let mut proof = SP1Prover::prove(ELF, stdin).expect("proving failed");
    // let mut proof_test = SP1Prover::prove(TEST_ELF,  SP1Stdin::new()).expect("proving failed");

    // // Read output.
    // let a = proof.public_values.read::<u128>();
    // let b = proof.public_values.read::<u128>();
    // println!("a: {}", a);
    // println!("b: {}", b);

    // // Verify proof.
    // SP1Verifier::verify(ELF, &proof).expect("verification failed");
    // SP1Verifier::verify(ELF, &proof_test).expect("verification failed");


    // println!("successfully generated and verified proof for the program!")
}
