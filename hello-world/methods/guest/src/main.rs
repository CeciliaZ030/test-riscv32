#![no_main]
use risc0_zkvm::guest::env;
risc0_zkvm::guest::entry!(main);


fn main() {
    let input: u32 = env::read();
    assert_eq!(1, 2);
    env::commit(&input);
}

// use k256 as risc0_k256;

// #[cfg(test)]
// mod test {

//     #[test]
//     fn fib() {
//         let mut a = 1;
//         let mut b = 1;
//         for _ in 0..10 {
//             let c = a + b;
//             a = b;
//             b = c;
//         }
//         assert_eq!(b, 144);
//     }
// }

