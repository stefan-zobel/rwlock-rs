use crate::owner::Owner;
use parking_lot::lock_api::{RwLockReadGuard, RwLockWriteGuard};
use parking_lot::{RawRwLock, RwLock};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;

#[derive(Debug)]
pub struct Lock<T: ?Sized> {
    // the whole point of the `owner` field is to avoid
    // that a writer accidentally can deadlock himself
    owner: Owner,
    rw_lock: RwLock<T>,
}

unsafe impl<T: ?Sized + Send> Send for Lock<T> {}
unsafe impl<T: ?Sized + Send + Sync> Sync for Lock<T> {}

impl<T> Lock<T> {
    #[inline]
    pub fn new(val: T) -> Lock<T> {
        Lock {
            rw_lock: RwLock::new(val),
            owner: Default::default(),
        }
    }

    #[inline]
    pub fn is_locked(&self) -> bool {
        self.rw_lock.is_locked()
    }

    #[inline]
    pub fn is_owned_by_current_thread(&self) -> bool {
        self.is_locked() && self.owner.is_current_thread()
    }

    #[inline]
    pub fn try_read_shared(&self) -> Option<ReadGuard<'_, T>> {
        if let Some(lock) = self.rw_lock.try_read_recursive() {
            assert!(!self.owner.is_owned());
            return Some(ReadGuard { guard: lock });
        }
        None
    }

    #[inline]
    pub fn read_shared(&self) -> Result<ReadGuard<'_, T>, LockError> {
        // avoid writer thread deadlocking itself by calling
        // `read_shared` after having obtained an exclusive
        // write lock
        if self.is_owned_by_current_thread() {
            return Err(LockError {});
        }
        let lock = self.rw_lock.read_recursive();
        assert!(!self.owner.is_owned());
        Ok(ReadGuard { guard: lock })
    }

    #[inline]
    pub fn try_write_exclusive(&self) -> Option<WriteGuard<'_, T>> {
        if let Some(lock) = self.rw_lock.try_write() {
            self.owner.take_ownership();
            return Some(WriteGuard {
                lock: self,
                guard: lock,
            });
        }
        None
    }

    #[inline]
    pub fn write_exclusive(&self) -> Result<WriteGuard<'_, T>, LockError> {
        // avoid writer thread deadlocking itself by calling
        // `write_exclusive` twice in a row
        if self.is_owned_by_current_thread() {
            return Err(LockError {});
        }
        let lock = self.rw_lock.write();
        self.owner.take_ownership();
        Ok(WriteGuard {
            lock: self,
            guard: lock,
        })
    }
}

#[derive(Debug)]
#[must_use]
pub struct WriteGuard<'a, T: ?Sized> {
    lock: &'a Lock<T>,
    guard: RwLockWriteGuard<'a, RawRwLock, T>,
}

impl<T: ?Sized> Drop for WriteGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.owner.release_ownership();
    }
}

#[derive(Debug)]
#[must_use]
pub struct ReadGuard<'a, T: ?Sized> {
    guard: RwLockReadGuard<'a, RawRwLock, T>,
}

impl<T> Deref for WriteGuard<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        #[cfg(test)]
        {
            assert!(self.lock.is_owned_by_current_thread());
        }
        self.guard.deref()
    }
}

impl<T> Deref for ReadGuard<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.guard.deref()
    }
}

pub struct LockError {}

impl Error for LockError {}

impl Debug for LockError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt("LockError", f)
    }
}

impl Display for LockError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt("lock already exclusively owned by current thread", f)
    }
}
