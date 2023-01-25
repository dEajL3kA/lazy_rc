/*
 * lazy_rc - Rc<T> and Arc<T> with *lazy* initialization
 * This is free and unencumbered software released into the public domain.
 */
use std::sync::{Arc, RwLock};

use crate::utils::{or_init_with, or_try_init_with};

/// A thread-safe reference-counting pointer, akin to
/// [`Arc<T>`](std::sync::Arc), but with ***lazy*** initialization
#[derive(Debug)]
pub struct LazyArc<T> {
    inner: RwLock<Option<Arc<T>>>,
}

impl<T> LazyArc<T> {
    /// Create a new `LazyArc<T>` that is initially *empty*. It's "inner" value
    /// will be [initialized](Self::or_init_with()) on first access!
    pub const fn empty() -> Self {
        Self {
            inner: RwLock::new(None)
        }
    }

    /// Create a new `LazyArc<T>` that is already initialized to `value`.
    pub fn from_value(value: T) -> Self {
        Self {
            inner: RwLock::new(Some(Arc::new(value)))
        }
    }

    /// Create a new `LazyArc<T>` that is already initialized to the given
    /// `Arc<T>` pointer.
    pub fn from_pointer(pointer: Arc<T>) -> Self {
        Self {
            inner: RwLock::new(Some(pointer))
        }
    }

    /// Returns `true`, if and only if th "inner" value is initialized.
    pub fn is_initialized(&self) -> bool {
        self.inner.read().map(|val| val.is_some()).unwrap_or(false)
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
        match self.value() {
            Some(value) => value,
            None => or_init_with(self.inner.write().unwrap(), || Arc::new(init_fn()))
        }
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
        match self.value() {
            Some(value) => Ok(value),
            None => or_try_init_with(self.inner.write().unwrap(), || init_fn().map(Arc::new))
        }
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
        self.inner.read().unwrap().as_ref().map(map_fn)
    }

    /// Returns a pointer to the "inner" value, if already initialized.
    /// 
    /// If and only if the "inner" value already *is* initialized, the function
    /// returns a new `Arc<T>` pointer to the "inner" value. Otherwise, if the
    /// "inner" value is **not** initialized yet, the value remains in the
    /// *uninitialized* state and the function returns `None`.
    pub fn value(&self) -> Option<Arc<T>> {
        self.inner.read().unwrap().as_ref().cloned()
    }

    /// Takes the "inner" value out of this `LazyRc<T>` instance, if already
    /// initialized.
    ///
    /// If and only if the "inner" value already *is* initialized, the function
    /// returns the `Arc<T>` pointer to the current "inner" value and resets
    /// this `LazyArc<T>` instance' "inner" value to the *uninitialized* state.
    /// Otherwise, the function simply returns `None`.
    pub fn take(&mut self) -> Option<Arc<T>> {
        self.inner.get_mut().unwrap().take()
    }
}

impl <T> Default for LazyArc<T> {
    /// The default value is a new ***empty*** `LazyArc<T>` instance.
    fn default() -> Self {
        Self::empty()
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
        match self.inner.read().unwrap().as_ref() {
            Some(existing) => Self::from_pointer(existing.clone()),
            _ => Self::empty(),
        }
    }
}
