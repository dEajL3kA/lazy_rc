/*
 * lazy_rc - Rc<T> and Arc<T> with *lazy* initialization
 * This is free and unencumbered software released into the public domain.
 */
use std::cell::RefCell;
use std::sync::Arc;

use crate::shared::or_init_with;

/// A thread-safe reference-counting pointer, akin to
/// [`Arc<T>`](std::sync::Arc), but with ***lazy*** initialization
#[derive(Debug)]
pub struct LazyArc<T> {
    inner: RefCell<Option<Arc<T>>>,
}

impl<T> LazyArc<T> {
    /// Create a new `LazyArc<T>` that is initially *empty*. It's "inner" value
    /// will be [initialized](Self::or_init_with()) on first access!
    pub fn empty() -> Self {
        Self {
            inner: RefCell::new(None)
        }
    }

    /// Create a new `LazyArc<T>` that is already initialized to `value`.
    pub fn from_value(value: T) -> Self {
        Self {
            inner: RefCell::new(Some(Arc::new(value)))
        }
    }

    /// Create a new `LazyArc<T>` that is already initialized to the given
    /// `Arc<T>` pointer.
    pub fn from_pointer(pointer: Arc<T>) -> Self {
        Self {
            inner: RefCell::new(Some(pointer))
        }
    }

    /// Returns a pointer to the existing "inner" value, or initializes the
    /// value right now.
    /// 
    /// If and only if the "inner" value is **not** initialized yet, the
    /// function `init_fn()` is called to create the value. The "inner" value
    /// is then set to the return value of `init_fn()` and a new `Arc<T>`
    /// pointer to the "inner" value is returned.
    pub fn or_init_with<F>(&self, init_fn: F) -> Arc<T>
    where
        F: FnOnce() -> T
    {
        self.or_try_init_with::<(), _>(|| Ok(init_fn())).unwrap()
    }

    /// Returns a pointer to the existing "inner" value, or tries to
    /// initializes the value right now.
    /// 
    /// If and only if the "inner" value is **not** initialized yet, the
    /// function `init_fn()` is called to create the value. In case that
    /// `init_fn()` returns an error, that error is passed through and the
    /// "inner" value remains in the *uninitialized* state for now. If the
    /// "inner" value already existed or if it was created successfully just
    /// now, a new `Arc<T>` pointer to the "inner" value is returned.
    pub fn or_try_init_with<E, F>(&self, init_fn: F) -> Result<Arc<T>, E>
    where
        F: FnOnce() -> Result<T, E>
    {
        or_init_with(self.inner.borrow_mut(), || init_fn().map(Arc::new))
    }

    /// Applies function `map_fn()` to the "inner", if already initialized.
    /// 
    /// If and only if the "inner" value already *is* initialize, the function
    /// `map_fn()` is called with a reference to the "inner" value and its
    /// return value is passed through. Otherwise the function `map_fn()` is
    /// **not** called and `None` is returned.
    pub fn map<U, F>(&self, map_fn: F) -> Option<U>
    where
        F: FnOnce(&Arc<T>) -> U
    {
        self.inner.borrow().as_ref().map(map_fn)
    }

    /// Returns a pointer to the "inner" value, if already initialized.
    /// 
    /// If and only if the "inner" value already *is* initialized, the function
    /// returns a new `Arc<T>` pointer to the "inner" value. Otherwise, if the
    /// "inner" value is **not** initialized yet, the value remains in the
    /// *uninitialized* state and the function returns `None`.
    pub fn value(&self) -> Option<Arc<T>> {
        self.inner.borrow().as_ref().map(|value| value.clone())
    }

    /// Takes the "inner" value out of this `LazyRc<T>` instance, if already
    /// initialized.
    ///
    /// If and only if the "inner" value already *is* initialized, the function
    /// returns the `Arc<T>` pointer to the current "inner" value and resets
    /// this `LazyArc<T>` instance' "inner" value to the *uninitialized* state.
    /// Otherwise, the function simply returns `None`.
    pub fn take(&mut self) -> Option<Arc<T>> {
        self.inner.get_mut().take()
    }
}

impl<T> Clone for LazyArc<T> {
    /// Creates a clone of this `LazyArc<T>` instance.
    /// 
    /// If the "inner" value of this instance *is* already initialized, the
    /// clone will be pointing to the same "inner" value, i.e. the "inner"
    /// value is **not** cloned. Otherwise, the clone will initially be
    /// *empty*; it can be initialized ***independently*** from this instance.
    fn clone(&self) -> LazyArc<T> {
        match self.inner.borrow().as_ref() {
            Some(existing) => Self::from_pointer(existing.clone()),
            _ => Self::empty(),
        }
    }
}
