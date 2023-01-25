/*
 * lazy_rc - Rc<T> and Arc<T> with *lazy* initialization
 * This is free and unencumbered software released into the public domain.
 */
use std::fmt::Debug;
use std::io::Result;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use chrono::{Timelike, Utc};
use rand::random;

use lazy_rc::LazyArc;

static GLOBAL_INSTANCE: LazyArc<MyStruct> = LazyArc::empty();

#[derive(Debug)]
struct MyStruct {
    _value: u32,
}

impl MyStruct {
    fn new() -> Result<Self> {
        // Simulate a lengthy computation!
        thread::sleep(Duration::from_secs(15));
        Ok(Self { _value: random() })
    }

    /// Returns a global instance that will be created on first access.
    /// If the initialization function fails, then an Error will be returned.
    pub fn instance() -> Result<Arc<Self>> {
        GLOBAL_INSTANCE.or_try_init_with(Self::new)
    }
}

fn main() {
    let mut threads = Vec::with_capacity(3);
    for _n in 0..3 {
        threads.push(thread::spawn(|| {
            print_with_timestamp("Start!");
            print_with_timestamp(MyStruct::instance());
            print_with_timestamp(MyStruct::instance());
            print_with_timestamp(MyStruct::instance());
        }));
    }
    threads.drain(..)
        .for_each(|t| t.join().unwrap());
}

fn print_with_timestamp<T>(value: T)
where
    T: Debug,
{
    let now = Utc::now();
    println!(
        "[{:02}:{:02}:{:02}.{:03}] [{:?}] {:?}",
        now.hour(),
        now.minute(),
        now.second(),
        now.nanosecond() / 1000000,
        thread::current().id(),
        value
    );
}
