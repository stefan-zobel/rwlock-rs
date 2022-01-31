#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

mod lock;
mod owner;

pub use lock::Lock as Lock;
pub use lock::ReadGuard as ReadGuard;
pub use lock::WriteGuard as WriteGuard;

#[cfg(test)]
mod tests {
    use crate::lock::Lock;

    #[test]
    fn test_debug_output() {
        let l = Lock::new(42);
        let _guard = l.write_exclusive();
        println!("Lock: {:?}", l);
    }
}
