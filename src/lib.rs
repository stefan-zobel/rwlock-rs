#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

mod lock;
mod owner;

pub use self::lock::Lock as Lock;
pub use self::lock::ReadGuard as ReadGuard;
pub use self::lock::WriteGuard as WriteGuard;

#[cfg(test)]
mod tests {
    use crate::lock::Lock;

    #[test]
    fn test_debug_output() {
        let l = Lock::new(42);
        let _guard = l.write_exclusive();
        println!("Lock: {:?}", l);
    }

    #[test]
    fn test_read_then_deadlock() {
        let l = Lock::new(42);
        let guard = l.read_shared().expect("read failed");
        println!("value: {}", *guard);
        l.try_write_exclusive();
        // now, provoke deadlock
//        l.write_exclusive();
    }
}
