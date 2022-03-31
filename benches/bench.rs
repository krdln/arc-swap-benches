#![feature(test, scoped_threads)]

extern crate test;

use arc_swap::ArcSwap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering::Relaxed};
use std::sync::Mutex;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;
use aligned::{Aligned, A64};

// type Thing = Vec<u8>;
// fn create() -> Thing { vec![1, 2, 3, 4, 5] }
// fn process(x: &Thing) -> u8 { x.iter().sum() }

use std::collections::HashMap;
type Thing = Aligned<A64, std::collections::HashMap<String, u8>>;
fn create() -> Thing { Aligned(HashMap::from_iter([("a".to_owned(), 5)])) }
fn process(x: &Thing) -> u8 { x["a"] }

use test::Bencher;

#[bench]
fn mutex_4(b: &mut Bencher) {
    let m = Mutex::new(Arc::new(create()));
    bench_par(b, 4, move || process(&m.lock().unwrap()));
}

#[bench]
fn mutex_unconteded(b: &mut Bencher) {
    let m = Mutex::new(Arc::new(create()));
    bench_par(b, 1, move || process(&m.lock().unwrap()));
}

#[bench]
fn rwlock_std_4(b: &mut Bencher) {
    let m = RwLock::new(Arc::new(create()));
    bench_par(b, 4, move || process(&m.read().unwrap()));
}

#[bench]
fn rwlock_std_uncontended(b: &mut Bencher) {
    let m = RwLock::new(Arc::new(create()));
    bench_par(b, 1, move || process(&m.read().unwrap()));
}

#[bench]
fn rwlock_parking_4(b: &mut Bencher) {
    let m = parking_lot::RwLock::new(Arc::new(create()));
    bench_par(b, 4, move || process(&m.read()));
}

#[bench]
fn rwlock_parking_uncontended(b: &mut Bencher) {
    let m = parking_lot::RwLock::new(Arc::new(create()));
    bench_par(b, 1, move || process(&m.read()));
}

#[bench]
fn rwlock_fast_4(b: &mut Bencher) {
    let m = fast::RwLock::new(Arc::new(create()));
    bench_par(b, 4, move || process(&m.read()));
}

#[bench]
fn rwlock_fast_uncontended(b: &mut Bencher) {
    let m = fast::RwLock::new(Arc::new(create()));
    bench_par(b, 1, move || process(&m.read()));
}

#[bench]
fn arcswap(b: &mut Bencher) {
    let m = ArcSwap::from_pointee(create());
    bench_par(b, 4, move || process(&m.load()));
}

#[bench]
fn arcswap_full(b: &mut Bencher) {
    let m = ArcSwap::from_pointee(create());
    bench_par(b, 4, move || process(&m.load_full()));
}

#[bench]
fn baseline(b: &mut Bencher) {
    let x = Arc::new(create());
    b.iter(|| process(test::black_box(&x)));
}

fn bench_par<R: 'static>(
    b: &mut Bencher,
    threads: usize,
    f: impl Fn() -> R + Sync + 'static,
) {
    assert!(threads != 0);
    let running = AtomicBool::new(true);
    let started = AtomicUsize::new(0);
    thread::scope(|s| {
        for _ in 1..threads {
            s.spawn(|| {
                started.fetch_add(1, Relaxed);
                while running.load(Relaxed) {
                    test::black_box(f());
                }
            });
        }
        while started.load(Relaxed) != threads - 1 {}
        b.iter(|| f());
        running.store(false, Relaxed);
    });
}

mod fast {
    use std::cell::UnsafeCell;
    use std::sync::atomic::AtomicI32;
    use std::sync::atomic::Ordering;

    /// A non-functional RwLock to see how would a golang-style RwLock perform.
    ///
    /// Golang RWMutex _unconditionally_ performs atomic add/sub on RLock() only then checks if the
    /// lock succeeded. This makes the contended rwlock with no writers quite fast.
    pub struct RwLock<T> {
        readers: AtomicI32,
        value: UnsafeCell<T>,
    }

    unsafe impl<T: Send> Send for RwLock<T> {}
    unsafe impl<T: Send + Sync> Sync for RwLock<T> {}

    pub struct Guard<'a, T> {
        lock: &'a RwLock<T>,
    }

    impl<T> RwLock<T> {
        pub fn new(val: T) -> Self {
            RwLock {
                /// Positive – num of readers, negative – special state.
                readers: AtomicI32::new(0),
                value: UnsafeCell::new(val),
            }
        }

        pub fn read(&self) -> Guard<T> {
            let prev_readers = self.readers.fetch_add(1, Ordering::Acquire);
            if prev_readers < 0 {
                todo!()
            }
            Guard { lock: self }
        }
    }

    impl<T> std::ops::Deref for Guard<'_, T> {
        type Target = T;
        fn deref(&self) -> &T {
            unsafe {
                &*self.lock.value.get()
            }
        }
    }

    impl<T> Drop for Guard<'_, T> {
        fn drop(&mut self) {
            let prev_state = self.lock.readers.fetch_sub(1, Ordering::Release);
            if prev_state < 0 {
                todo!()
            }
        }
    }
}
