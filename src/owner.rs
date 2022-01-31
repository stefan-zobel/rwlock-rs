use parking_lot::lock_api::GetThreadId;
use parking_lot::RawThreadId;
use std::fmt::{Debug, Formatter};
use std::sync::atomic::{AtomicU64, Ordering};

pub(crate) struct Owner {
    owner: AtomicU64,
    get_thread_id: RawThreadId,
}

impl Owner {
    #[inline]
    pub(crate) fn is_current_thread(&self) -> bool {
        let id = self.owner.load(Ordering::Relaxed);
        if id != 0 && id == self.current_thread_id() {
            return true;
        }
        false
    }

    #[inline]
    pub(crate) fn take_ownership(&self) {
        assert!(!self.is_owned());
        self.owner
            .store(self.current_thread_id(), Ordering::Relaxed);
    }

    #[inline]
    pub(crate) fn release_ownership(&self) {
        assert!(self.is_current_thread());
        self.owner.store(0, Ordering::Relaxed);
    }

    #[inline]
    pub(crate) fn is_owned(&self) -> bool {
        self.owner.load(Ordering::Relaxed) != 0
    }

    #[inline]
    pub(crate) fn new() -> Owner {
        Owner {
            owner: AtomicU64::new(0),
            get_thread_id: RawThreadId,
        }
    }

    #[inline]
    fn current_thread_id(&self) -> u64 {
        self.get_thread_id.nonzero_thread_id().get() as u64
    }
}

impl Default for Owner {
    #[inline]
    fn default() -> Self {
        Owner::new()
    }
}

impl Debug for Owner {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.owner.load(Ordering::Relaxed))
    }
}
