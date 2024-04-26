#![no_main]
// If you want to try std support, also update the guest Cargo.toml file
// #![no_std]  // std support is experimental


use risc0_zkvm::guest::env;

risc0_zkvm::guest::entry!(main);


fn main() {
    // TODO: Implement your guest code here

    // read the input
    let input: u32 = env::read();

    // TODO: do something with the input

    // write public output to the journal
    env::commit(&input);
}

// use k256 as risc0_k256;

#[cfg(test)]
mod test {

    #[test]
    fn fib() {
        let mut a = 1;
        let mut b = 1;
        for _ in 0..10 {
            let c = a + b;
            a = b;
            b = c;
        }
        assert_eq!(b, 144);
    }
}

