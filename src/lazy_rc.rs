/*
 * lazy_rc - Rc<T> and Arc<T> with *lazy* initialization
 * This is free and unencumbered software released into the public domain.
 */
use std::fmt::Debug;
use std::io::{Result as IoResult};
use std::rc::Rc;
use std::cell::RefCell;

use crate::InitError;
use crate::utils::{DefaultInit, or_init_with, or_try_init_with};

/// A single-threaded reference-counting pointer, akin to
/// [`Rc<T>`](std::rc::Rc), but with ***lazy*** initialization
pub struct LazyRc<T> {
    inner: RefCell<Option<Rc<T>>>,
    default_init: DefaultInit<T>,
}

impl<T> LazyRc<T> {
    /// Create a new `LazyRc<T>` that is initially *empty* and that contains
    /// **no** *default* initializer.
    /// 
    /// The "inner" value will be [initialized](Self::or_init_with()) on first
    /// access. Default initialization is **not** supported by this instance!
    pub const fn empty() -> Self {
        Self {
            inner: RefCell::new(None),
            default_init: DefaultInit::None,
        }
    }

    /// Create a new `LazyRc<T>` that is initially *empty* and that contains
    /// the given *default* initializer.
    /// 
    /// The "inner" value will be [initialized](Self::or_init_with()) on first
    /// access. Default initialization *is* supported by this instance.
    pub fn with_default_init<U>(default_init: U) -> Self
    where
        U: Fn() -> T + Sync + 'static,
    {
        Self {
            inner: RefCell::new(None),
            default_init: DefaultInit::Infailable(Box::new(default_init)),
        }
    }

    /// Create a new `LazyRc<T>` that is initially *empty* and that contains
    /// the given failable *default* initializer.
    /// 
    /// The "inner" value will be [initialized](Self::or_init_with()) on first
    /// access. Default initialization *is* supported by this instance.
    pub fn with_failable_default_init<U>(default_init: U) -> Self
    where
        U: Fn() -> IoResult<T> + Sync + 'static,
    {
        Self {
            inner: RefCell::new(None),
            default_init: DefaultInit::Failable(Box::new(default_init)),
        }
    }

    /// Returns `true`, if and only if th "inner" value is initialized.
    pub fn is_initialized(&self) -> bool {
        self.inner.borrow().is_some()
    }

    /// Returns a pointer to the existing "inner" value, or tries to initialize
    /// the value right now.
    /// 
    /// If and only if the "inner" value is **not** initialized yet, the
    /// "inner" value is set to the return value of the *default* initializer
    /// and a new `Rc<T>` pointer to the "inner" value is returned. The
    /// default initializer **must** be *infailable*, otherwise use
    /// [`or_try_init()`](Self::or_try_init)!
    /// 
    /// Warning: This function [panics](mod@std::panic), if **no** *default*
    /// initializer is available, or of the default initializer is *failable*!
    pub fn or_init(&self) -> Rc<T> {
        match &self.default_init {
            DefaultInit::Infailable(init) => or_init_with(self.inner.borrow_mut(), || Rc::new(init())),
            _ => panic!("No infailable default initializer!"),
        }
    }

    /// Returns a pointer to the existing "inner" value, or tries to initialize
    /// the value right now.
    /// 
    /// If and only if the "inner" value is **not** initialized yet, the
    /// "inner" value is set to the return value of the *default* initializer
    /// and a new `Rc<T>` pointer to the "inner" value is returned. If the
    /// *default* initializer fails, the error is passed through.
    /// 
    /// If **no** *default* initializer is available, an error of type
    /// [`NoDefaultInitializer`](crate::InitError) is returned.
    pub fn or_try_init(&self) -> Result<Rc<T>, InitError> {
        match &self.default_init {
            DefaultInit::Infailable(init) => Ok(or_init_with(self.inner.borrow_mut(), || Rc::new(init()))),
            DefaultInit::Failable(init) => match or_try_init_with(self.inner.borrow_mut(), || init().map(Rc::new)) {
                Ok(value) => Ok(value),
                Err(error) => Err(InitError::Failed(error)),
            },
            DefaultInit::None => Err(InitError::NoDefaultInitializer)
        }
    }

    /// Returns a pointer to the existing "inner" value, or initializes the
    /// value right now.
    /// 
    /// If and only if the "inner" value is **not** initialized yet, the
    /// function `init_fn()` is called to create the value. The "inner" value
    /// is then set to the return value of `init_fn()` and a new `Rc<T>`
    /// pointer to the "inner" value is returned.
    pub fn or_init_with<F>(&self, init_fn: F) -> Rc<T>
    where
        F: FnOnce() -> T
    {
        or_init_with(self.inner.borrow_mut(), || Rc::new(init_fn()))
    }

    /// Returns a pointer to the existing "inner" value, or tries to
    /// initializes the value right now.
    /// 
    /// If and only if the "inner" value is **not** initialized yet, the
    /// function `init_fn()` is called to create the value. In case that
    /// `init_fn()` returns an error, that error is passed through and the
    /// "inner" value remains in the *uninitialized* state for now. If the
    /// "inner" value already existed or if it was created successfully just
    /// now, a new `Rc<T>` pointer to the "inner" value is returned.
    pub fn or_try_init_with<E, F>(&self, init_fn: F) -> Result<Rc<T>, E>
    where
        F: FnOnce() -> Result<T, E>
    {
        or_try_init_with(self.inner.borrow_mut(), || init_fn().map(Rc::new))
    }

    /// An alias for the [`or_init()`](Self::or_init) function.
    pub fn unwrap(&self) -> Rc<T> {
        self.or_init()
    }

    /// Applies function `map_fn()` to the "inner", if already initialized.
    /// 
    /// If and only if the "inner" value already *is* initialize, the function
    /// `map_fn()` is called with a reference to the "inner" value and its
    /// return value is passed through. Otherwise the function `map_fn()` is
    /// **not** called and `None` is returned.
    pub fn map<U, F>(&self, map_fn: F) -> Option<U>
    where
        F: FnOnce(&Rc<T>) -> U
    {
        self.inner.borrow().as_ref().map(map_fn)
    }

    /// Returns a pointer to the "inner" value, if already initialized.
    /// 
    /// If and only if the "inner" value already *is* initialized, the function
    /// returns a new `Rc<T>` pointer to the "inner" value. Otherwise, if the
    /// "inner" value is **not** initialized yet, the value remains in the
    /// *uninitialized* state and the function returns `None`.
    pub fn value(&self) -> Option<Rc<T>> {
        self.inner.borrow().as_ref().cloned()
    }

    /// Takes the "inner" value out of this `LazyRc<T>` instance, if already
    /// initialized.
    ///
    /// If and only if the "inner" value already *is* initialized, the function
    /// returns the `Rc<T>` pointer to the current "inner" value and resets
    /// this `LazyRc<T>` instance' "inner" value to the *uninitialized* state.
    /// Otherwise, the function simply returns `None`.
    pub fn take(&mut self) -> Option<Rc<T>> {
        self.inner.get_mut().take()
    }
}

impl <T> Default for LazyRc<T> {
    /// The default value is a new ***empty*** `LazyRc<T>` instance.
    fn default() -> Self {
        Self::empty()
    }
}

impl <T> From<T> for LazyRc<T> {
    /// Create a new `LazyRc<T>` that is already initialized to `value`.
    fn from(value: T) -> Self {
        Self {
            inner: RefCell::new(Some(Rc::new(value))),
            default_init: DefaultInit::None,
        }
    }
}

impl <T> From<&T> for LazyRc<T>
where
    T: Clone,
{
    /// Create a new `LazyRc<T>` that is already initialized to `value`.
    fn from(value: &T) -> Self {
        Self {
            inner: RefCell::new(Some(Rc::new(value.clone()))),
            default_init: DefaultInit::None,
        }
    }
}

impl <T> From<Rc<T>> for LazyRc<T> {
    /// Create a new `LazyRc<T>` that is already initialized to `value`.
    fn from(value: Rc<T>) -> Self {
        Self {
            inner: RefCell::new(Some(value)),
            default_init: DefaultInit::None,
        }
    }
}

impl <T> From<&Rc<T>> for LazyRc<T> {
    /// Create a new `LazyRc<T>` that is already initialized to `value`.
    fn from(value: &Rc<T>) -> Self {
        Self {
            inner: RefCell::new(Some(value.clone())),
            default_init: DefaultInit::None,
        }
    }
}

impl<T> Clone for LazyRc<T> {
    /// Creates a clone of this `LazyRc<T>` instance.
    /// 
    /// If the "inner" value of this instance *is* already initialized, the
    /// clone will be pointing to the same "inner" value, i.e. the "inner"
    /// value is **not** cloned. Otherwise, the clone will initially be
    /// *empty*; it can be initialized ***independently*** from this instance.
    fn clone(&self) -> LazyRc<T> {
        match self.inner.borrow().as_ref() {
            Some(existing) => Self::from(existing),
            _ => Self::empty(),
        }
    }
}

impl<T> Debug for LazyRc<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LazyRc {{ default_init: {:?}, is_initialized: {:?} }}",
            self.default_init,
            self.inner.borrow().is_some())
    }
}
