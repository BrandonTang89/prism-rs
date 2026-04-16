use std::{marker::PhantomData, rc::Rc};

use sylvan_sys::mtbdd::{Sylvan_mtbdd_refs_popptr, Sylvan_mtbdd_refs_pushptr};
use sylvan_sys::MTBDD;

pub struct LocalRootsGuard {
    pushed: usize,
    // Make it neither Send nor Sync: the Sylvan local ref stack is per-thread.
    _not_send_sync: PhantomData<Rc<()>>,
}

impl LocalRootsGuard {
    pub fn new() -> Self {
        Self {
            pushed: 0,
            _not_send_sync: PhantomData,
        }
    }

    /// Root a local MTBDD variable by address.
    ///
    /// # Safety
    /// - `ptr` must point to a valid `MTBDD` variable.
    /// - That variable must remain alive and at the same address until this guard is dropped.
    /// - The guard must be dropped on the same thread it was created on.
    pub unsafe fn push_raw(&mut self, ptr: *const MTBDD) {
        unsafe { Sylvan_mtbdd_refs_pushptr(ptr) };
        self.pushed += 1;
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.pushed
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.pushed == 0
    }
}

impl Drop for LocalRootsGuard {
    fn drop(&mut self) {
        if self.pushed != 0 {
            unsafe {
                Sylvan_mtbdd_refs_popptr(self.pushed);
            }
        }
    }
}
