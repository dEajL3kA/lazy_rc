/*
 * lazy_rc - Rc<T> and Arc<T> with *lazy* initialization
 * This is free and unencumbered software released into the public domain.
 */
use std::error::Error;
use std::fmt::Debug;
use std::io::{Error as IoError, Result as IoResult};
use std::fmt::Display;
use std::ops::DerefMut;

type FnInit<T> = dyn Fn() -> T + Sync;
type FnInitFailable<T> = dyn Fn() -> IoResult<T> + Sync;

/// A wrapper that optionally contains a (possibly failable) initializer.
pub enum DefaultInit<T> {
    None,
    Infailable(Box<FnInit<T>>),
    Failable(Box<FnInitFailable<T>>),
}

/// An error that indicates that the initialization has failed.
#[derive(Debug)]
pub enum InitError {
    /// Initialization failed, because **no** default initializer is available!
    NoDefaultInitializer,
    /// The initializer function has failed! The original error is forwarded as
    /// "inner" value of this [`InitError`] variant.
    Failed(IoError),
}

pub fn or_init_with<T, F>(mut inner: impl DerefMut<Target = Option<T>>, init_fn: F) -> T
where
    T: Clone,
    F: FnOnce() -> T,
{
    match inner.as_ref() {
        Some(existing) => existing.clone(),
        None => inner.insert(init_fn()).clone(),
    }
}

pub fn or_try_init_with<T, E, F>(
    mut inner: impl DerefMut<Target = Option<T>>,
    init_fn: F,
) -> Result<T, E>
where
    T: Clone,
    F: FnOnce() -> Result<T, E>,
{
    match inner.as_ref() {
        Some(existing) => Ok(existing.clone()),
        None => match init_fn() {
            Ok(value) => Ok(inner.insert(value).clone()),
            Err(error) => Err(error),
        },
    }
}

impl<T> Debug for DefaultInit<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Infailable(_) => write!(f, "Infailable"),
            Self::Failable(_) => write!(f, "Failable"),
        }
    }
}

impl Display for InitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InitError::NoDefaultInitializer => write!(f, "No default initializer available!"),
            InitError::Failed(error) => Display::fmt(&error, f),
        }
    }
}

impl Error for InitError {}
