#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

mod hash;
mod lock;
mod owner;

pub use self::lock::Lock;
pub use self::lock::ReadGuard;
pub use self::lock::WriteGuard;

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
    fn test_read_once_then_attempt_write() {
        let l = Lock::new(42);
        let guard = l.read_shared().expect("read failed");
        println!("value: {}", *guard);
        let write_guard1 = l.try_write_exclusive();
        assert!(write_guard1.is_none());
        // now attempt an exclusive write
        let write_guard2 = l.write_exclusive();
        assert!(write_guard2.is_err());
    }

    #[test]
    fn test_read_twice_then_attempt_write() {
        let l = Lock::new(42);
        let read_guard1 = l.read_shared().expect("read failed");
        println!("first read: {}", *read_guard1);
        {
            let read_guard2 = l.read_shared().expect("read failed");
            println!("second read: {}", *read_guard2);
        }
        let write_guard1 = l.try_write_exclusive();
        assert!(write_guard1.is_none());
        // now attempt an exclusive write
        let write_guard2 = l.write_exclusive();
        assert!(write_guard2.is_err());
    }
}
