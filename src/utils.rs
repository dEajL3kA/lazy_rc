/*
 * lazy_rc - Rc<T> and Arc<T> with *lazy* initialization
 * This is free and unencumbered software released into the public domain.
 */
use std::ops::DerefMut;

pub fn or_init_with<T, F>(mut inner: impl DerefMut<Target=Option<T>>, init_fn: F) -> T
where
    T: Clone,
    F: FnOnce() -> T, 
{
    match inner.as_ref() {
        Some(existing) => existing.clone(),
        None => inner.insert(init_fn()).clone(),
    }
}

pub fn or_try_init_with<T, E, F>(mut inner: impl DerefMut<Target=Option<T>>, init_fn: F) -> Result<T, E>
where
    T: Clone,
    F: FnOnce() -> Result<T, E>, 
{
    match inner.as_ref() {
        Some(existing) => Ok(existing.clone()),
        None => match init_fn() {
            Ok(value) => Ok(inner.insert(value).clone()),
            Err(error) => Err(error),
        }
    }
}