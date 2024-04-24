//! A simple script to generate and verify the proof of a given program.
#![feature(test)]
use std::default;

use sp1_sdk::{ProverClient, SP1Prover, SP1Stdin};
use raiko_lib::{
    input::{GuestInput, GuestOutput, WrappedHeader},
};

const ELF: &[u8] = include_bytes!("../../program/elf/riscv32im-succinct-zkvm-elf");
const TEST_ELF: &[u8] = include_bytes!("../../program/elf/riscv32im-succinct-zkvm-elf-test");


extern crate test;
//  test_main(args: &[String], tests: Vec<TestDescAndFn>, options: Option<Options>)

fn main() {
    // Generate proof.
    let mut stdin = SP1Stdin::new();
    stdin.write::<u32>(&GuestInput::default());

    let mut stdint = SP1Stdin::new();
    stdint.write::<Vec<String>>(&Vec::new());

    // Generate the proof for the given program.
    let client = ProverClient::new();
    let mut proof = client.prove(TEST_ELF, stdint).expect("Sp1: proving failed");

    // Verify proof.
    client
        .verify(TEST_ELF, &proof)
        .expect("Sp1: verification failed");


    println!("successfully generated and verified proof for the program!")
}


// #[derive(Debug, serde::ser::Serialize, Default)]
// pub struct TestOpts {
//     pub list: bool,
//     pub filters: Vec<String>,
//     pub filter_exact: bool,
//     pub force_run_in_process: bool,
//     pub exclude_should_panic: bool,
//     pub run_ignored: RunIgnored,
//     pub run_tests: bool,
//     pub bench_benchmarks: bool,
//     pub logfile: Option<PathBuf>,
//     pub nocapture: bool,
//     pub color: test::ColorConfig,
//     pub format: test::OutputFormat,
//     pub shuffle: bool,
//     pub shuffle_seed: Option<u64>,
//     pub test_threads: Option<usize>,
//     pub skip: Vec<String>,
//     pub time_options: Option<test::time::TestTimeOptions>,
//     /// Stop at first failing test.
//     /// May run a few more tests due to threading, but will
//     /// abort as soon as possible.
//     pub fail_fast: bool,
//     pub options: Options,
// }

// /// Whether ignored test should be run or not
// #[derive(Copy, Clone, Debug, PartialEq, Eq)]
// pub enum RunIgnored {
//     Yes,
//     #[default]
//     No,
//     /// Run only ignored tests
//     Only,
// }

// fn new_test_opts() {
//     TestOpts {
//         list: false,
//         filters: Vec::new(),
//         filter_exact: false,
//         force_run_in_process: false,
//         exclude_should_panic: false,
//         run_ignored: test::RunIgnored::No,
//         run_tests: true,
//         bench_benchmarks: false,
//         logfile: None,
//         nocapture: false,
//         color: test::ColorConfig::AutoColor,
//         format: test::OutputFormat::Terse,
//         shuffle: false,
//         shuffle_seed: None,
//         test_threads: None,
//         skip: Vec::new(),
//         time_options: None,
//         fail_fast: false,
//         options: Options::new(),
//     }
// }