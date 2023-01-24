/*
 * lazy_rc - Rc<T> and Arc<T> with *lazy* initialization
 * This is free and unencumbered software released into the public domain.
 */
use std::cell::RefMut;

pub(crate) fn or_init_with<T, E, F>(mut inner: RefMut<Option<T>>, init_fn: F) -> Result<T, E>
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
