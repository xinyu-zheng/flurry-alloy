pub(crate) use seize::{Collector, Guard, Linked};

use std::gc::Gc;
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::{fmt, ptr};

pub(crate) struct Atomic<T>(AtomicPtr<Gc<T>>);

impl<T> Atomic<T> {
    pub(crate) fn null() -> Self {
        Self(AtomicPtr::default())
    }

    pub(crate) fn load<'g>(&self, ordering: Ordering) -> Shared<'g, T> {
        self.0.load(ordering).into()
    }

    pub(crate) fn store(&self, new: Shared<'_, T>, ordering: Ordering) {
        self.0.store(new.ptr, ordering);
    }

    pub(crate) unsafe fn into_box(self) -> Box<Gc<T>> {
        Box::from_raw(self.0.into_inner())
    }

    pub(crate) fn swap<'g>(&self, new: Shared<'_, T>, ord: Ordering) -> Shared<'g, T> {
        self.0.swap(new.ptr, ord).into()
    }

    pub(crate) fn compare_exchange<'g>(
        &self,
        current: Shared<'_, T>,
        new: Shared<'g, T>,
        success: Ordering,
        failure: Ordering,
    ) -> Result<Shared<'g, T>, CompareExchangeError<'g, T>> {
        match self
            .0
            .compare_exchange(current.ptr, new.ptr, success, failure)
        {
            Ok(ptr) => Ok(ptr.into()),
            Err(current) => Err(CompareExchangeError {
                current: current.into(),
                new,
            }),
        }
    }
}

impl<T> From<Shared<'_, T>> for Atomic<T> {
    fn from(shared: Shared<'_, T>) -> Self {
        Atomic(shared.ptr.into())
    }
}

impl<T> Clone for Atomic<T> {
    fn clone(&self) -> Self {
        Atomic(self.0.load(Ordering::Relaxed).into())
    }
}

impl<T> fmt::Debug for Shared<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:p}", self.ptr)
    }
}

impl<T> fmt::Debug for Atomic<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:p}", self.0.load(Ordering::SeqCst))
    }
}

pub(crate) struct CompareExchangeError<'g, T> {
    pub(crate) current: Shared<'g, T>,
    pub(crate) new: Shared<'g, T>,
}

pub(crate) struct Shared<'g, T> {
    ptr: *mut Gc<T>,
    _g: PhantomData<&'g ()>,
}

impl<'g, T> Shared<'g, T> {
    pub(crate) fn null() -> Self {
        Shared::from(ptr::null_mut())
    }

    pub(crate) fn boxed(value: T) -> Self {
        Shared::from(Box::into_raw(Box::new(Gc::new(value))))
    }

    pub(crate) unsafe fn into_box(self) -> Box<Gc<T>> {
        Box::from_raw(self.ptr)
    }

    pub(crate) unsafe fn as_ptr(&self) -> *mut Gc<T> {
        self.ptr
    }

    pub(crate) unsafe fn as_ref(&self) -> Option<&'g Gc<T>> {
        self.ptr.as_ref()
    }

    pub(crate) unsafe fn deref(&self) -> &'g Gc<T> {
        &*self.ptr
    }

    pub(crate) fn is_null(&self) -> bool {
        self.ptr.is_null()
    }
}

impl<'g, T> PartialEq<Shared<'g, T>> for Shared<'g, T> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr
    }
}

impl<T> Eq for Shared<'_, T> {}

impl<T> Clone for Shared<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Shared<'_, T> {}

impl<T> From<*mut Gc<T>> for Shared<'_, T> {
    fn from(ptr: *mut Gc<T>) -> Self {
        Shared {
            ptr,
            _g: PhantomData,
        }
    }
}
