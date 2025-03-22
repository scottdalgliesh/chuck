#![no_std]
#[cfg(test)]
mod tests {
    extern crate std;
    use std::println;

    #[test]
    fn sample_test() {
        println!("Sample test is running.");
        assert!(true);
    }
}
