/*
 * lazy_rc - Rc<T> and Arc<T> with *lazy* initialization
 * This is free and unencumbered software released into the public domain.
 */

//! **lazy_rc** provides implementations of [`Rc<T>`](std::rc::Rc) and
//! [`Arc<T>`](std::sync::Arc) with ***lazy*** initialization.
//! 
//! In other words, the "inner" value of an [**`LazyRc<T>`**](LazyRc) or
//! [**`LazyArc<T>`**](LazyArc) instance is created when it is accessed for the
//! *first* time, using the supplied initialization function. Initialization
//! may fail, in which case the error is passed through.
//! 
//! # Thread Safety
//! 
//! `LazyRc<T>` is *single-threaded*, because so is `Rc<T>`. Therefore, an
//! `LazyRc<T>` instance can **not** be shared by multiple threads, and you can
//! **not** use `LazyRc<T>` for **`static`** variables. However, it ***can***
//! be used for [`thread_local!`](std::thread_local) variables.
//! 
//! `LazyArc<T>` is *thread-safe*, because so is `Arc<T>`. Therefore, an
//! `LazyArc<T>` instance can be shared by multiple threads, and you can even
//! use `LazyArc<T>` for *global* **`static`** variables.
//! 
//! # Const Warning
//! 
//! Do **not** use `LazyRc<T>` or `LazyArc<T>` as a **`const`** value! That is
//! because, in Rust, **`const`** values are "inlined", effectively creating a
//! *new* instance at every place where the **`const`** value is used. This
//! obviously breaks "lazy" initialization 😨
//! 
//! # Example
//! 
//! ```
//! use lazy_rc::{LazyRc, LazyArc};
//! 
//! static GLOBAL_INSTANCE: LazyArc<MyStruct> = LazyArc::empty();
//! 
//! thread_local! {
//!     static THREAD_INSTANCE: LazyRc<MyStruct>  = LazyRc::empty();
//! }
//! 
//! struct MyStruct {
//!    /* ... */
//! }
//! 
//! impl MyStruct {
//!     fn new() -> Result<Self> {
//!         /* ... */
//!     }
//! 
//!     /// Returns a thread-local instance that will be created on first access.
//!     /// If the initialization function fails, then an Error will be returned.
//!     pub fn instance() -> Result<Rc<Self>> {
//!         THREAD_INSTANCE.with(|lazy| lazy.or_try_init_with(Self::new))
//!     }
//! }
//! ```

mod lazy_arc;
mod lazy_rc;

pub(crate) mod utils;

pub use lazy_arc::LazyArc;
pub use lazy_rc::LazyRc;
pub use utils::InitError;
