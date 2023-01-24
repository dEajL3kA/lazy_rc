/*
 * lazy_rc - Rc<T> and Arc<T> with *lazy* initialization
 * This is free and unencumbered software released into the public domain.
 */
use std::rc::Rc;
use std::thread;
use std::time::Duration;

use lazy_rc::LazyRc;

thread_local! {
    static INSTANCE: LazyRc<MyStruct>  = LazyRc::empty();
}

#[derive(Debug)]
struct MyStruct {
    _secret_value: u32,
}

impl MyStruct {
    fn new() -> Self {
        thread::sleep(Duration::from_secs(10)); // <-- simulate lengthy computation
        Self {
            _secret_value: 42,
        }
    }

    pub fn instance() -> Rc<Self> {
        INSTANCE.with(|instance| instance.or_init_with(Self::new))
    }
}

fn main() {
    let my_instance_1 = MyStruct::instance();
    println!("{:?}", my_instance_1);

    let my_instance_2 = MyStruct::instance();
    println!("{:?}", my_instance_2);

    let my_instance_3 = MyStruct::instance();
    println!("{:?}", my_instance_3);
}
