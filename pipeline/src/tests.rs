
const TEST_ELF: &[u8] = include_bytes!("../../program/elf/riscv32im-succinct-zkvm-elf-test");
use sp1_sdk::{ProverClient, SP1Prover, SP1Stdin};


#[test]
fn fibo() {
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
fn sp1_elf() {
    let mut stdint = SP1Stdin::new();
    stdint.write::<Vec<String>>(&Vec::new());

    // Generate the proof for the given program.
    let client = ProverClient::new();
    let mut proof = client.prove(TEST_ELF, stdint).expect("Sp1: proving failed");

    // Verify proof.
    client
        .verify(TEST_ELF, &proof)
        .expect("Sp1: verification failed");

}