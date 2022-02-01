use crate::hash::BytesHash;
use crate::owner::Owner;
use parking_lot::lock_api::{RwLockReadGuard, RwLockWriteGuard};
use parking_lot::{RawRwLock, RwLock};
use std::cell::UnsafeCell;
use std::collections::HashSet;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;

#[derive(Debug)]
pub struct Lock<T: ?Sized> {
    // the whole point of the `exclusive_owner` field is to
    // avoid that a writer accidentally can deadlock himself
    exclusive_owner: Owner,
    rw_lock: RwLock<T>,
}

unsafe impl<T: ?Sized + Send> Send for Lock<T> {}
unsafe impl<T: ?Sized + Send + Sync> Sync for Lock<T> {}

impl<T> Lock<T> {
    #[inline]
    pub fn new(val: T) -> Lock<T> {
        Lock {
            rw_lock: RwLock::new(val),
            exclusive_owner: Default::default(),
        }
    }

    #[inline]
    pub fn is_locked(&self) -> bool {
        self.rw_lock.is_locked()
    }

    #[inline]
    pub fn is_owned_by_current_thread(&self) -> bool {
        self.is_locked() && self.exclusive_owner.is_current_thread()
    }

    #[inline]
    pub fn try_read_shared(&self) -> Option<ReadGuard<'_, T>> {
        if let Some(lock) = self.rw_lock.try_read_recursive() {
            assert!(!self.exclusive_owner.is_owned());
            self.add_read_lock();
            return Some(ReadGuard {
                lock: self,
                guard: lock,
            });
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
        assert!(!self.exclusive_owner.is_owned());
        self.add_read_lock();
        Ok(ReadGuard {
            lock: self,
            guard: lock,
        })
    }

    #[inline]
    pub fn try_write_exclusive(&self) -> Option<WriteGuard<'_, T>> {
        if let Some(lock) = self.rw_lock.try_write() {
            self.exclusive_owner.take_ownership();
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
        if self.is_owned_by_current_thread() || self.is_read_lock_held() {
            return Err(LockError {});
        }
        let lock = self.rw_lock.write();
        self.exclusive_owner.take_ownership();
        Ok(WriteGuard {
            lock: self,
            guard: lock,
        })
    }

    #[inline]
    pub(crate) fn is_read_lock_held(&self) -> bool {
        let read_set = Lock::<T>::get_thread_local();
        unsafe { (*read_set).contains(&(self.get_address())) }
    }

    #[inline]
    pub(crate) fn add_read_lock(&self) {
        let read_set = Lock::<T>::get_thread_local();
        unsafe { (*read_set).insert(self.get_address()) };
    }

    #[inline]
    pub(crate) fn remove_read_lock(&self) {
        let read_set = Lock::<T>::get_thread_local();
        unsafe { (*read_set).remove(&(self.get_address())) };
    }

    #[inline]
    pub(crate) fn get_address(&self) -> u64 {
        self as *const Self as u64
    }

    #[inline]
    pub(crate) fn get_thread_local() -> *mut HashSet<u64, BytesHash> {
        THREAD_LOCAL.with(|t| t.get())
    }
}

#[derive(Debug)]
#[must_use]
pub struct WriteGuard<'a, T: ?Sized> {
    lock: &'a Lock<T>,
    guard: RwLockWriteGuard<'a, RawRwLock, T>,
}

impl<T> Drop for ReadGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.remove_read_lock();
    }
}

impl<T: ?Sized> Drop for WriteGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.exclusive_owner.release_ownership();
    }
}

#[derive(Debug)]
#[must_use]
pub struct ReadGuard<'a, T> {
    lock: &'a Lock<T>,
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

thread_local!(
    static THREAD_LOCAL: UnsafeCell<HashSet<u64, BytesHash>> = {
        UnsafeCell::new(HashSet::with_hasher(BytesHash::default()))
    }
);

pub struct LockError {}

impl Error for LockError {}

impl Debug for LockError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt("LockError", f)
    }
}

impl Display for LockError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt("a lock is already owned by the current thread", f)
    }
}
