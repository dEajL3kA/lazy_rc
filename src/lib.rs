/*
 * lazy_rc - Rc<T> and Arc<T> with *lazy* initialization
 * This is free and unencumbered software released into the public domain.
 */

//! **lazy_rc** provides implementations of [`Rc<T>`](std::rc::Rc) and
//! [`Arc<T>`](std::sync::Arc) with ***lazy*** initialization.
//! 
//! In other words, the "inner" value of an [`LazyRc<T>`](LazyRc) or
//! [`LazyArc<T>`](LazyArc) instance is created when it is accessed for the
//! *first* time, using the supplied initialization function. Initialization
//! may fail, in which case the error is passed through.
//! 
//! # Example
//! 
//! ```
//! use lazy_rc::LazyRc;
//! 
//! thread_local! {
//!     static INSTANCE: LazyRc<MyStruct>  = LazyRc::empty();
//! }
//! 
//! #[derive(Debug)]
//! struct MyStruct {
//!     /* ... */
//! }
//! 
//! impl MyStruct {
//!     fn new() -> Self {
//!         /* ... */
//!     }
//! 
//!     pub fn instance() -> Rc<Self> {
//!         INSTANCE.with(|instance| instance.or_init_with(Self::new))
//!     }
//! }
//! ```

mod lazy_rc;
mod lazy_arc;
mod shared;

pub use lazy_rc::LazyRc;
pub use lazy_arc::LazyArc;
