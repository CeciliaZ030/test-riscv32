
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]


pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}


pub fn add_two(a: i32) -> i32 {
    a + 2
}



pub fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
    println!("Test run complete.~~~~~~~~~~~~~");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_add_two() {
        assert_eq!(add_two(2), 4);
    }
}

