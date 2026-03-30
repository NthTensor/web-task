#![cfg(target_feature = "atomics")]

use core::cell::Cell;
use core::future::poll_fn;
use core::task::Poll;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;

use web_task::spawn_local;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = setTimeout)]
    fn set_timeout(closure: &js_sys::Function, ms: u32) -> i32;
}

async fn flush_event_loop() {
    let promise = js_sys::Promise::new(&mut |resolve, _| {
        set_timeout(&resolve, 0);
    });
    JsFuture::from(promise).await.unwrap();
}

/// Reproduces an error present in `wasm_bindgen_futures` at one point.
#[wasm_bindgen_test(async)]
async fn wait_async_promise_callback_runs_without_wake() {
    let mut polled = false;
    let future = poll_fn(move |cx| {
        if polled {
            Poll::Ready(42)
        } else {
            cx.waker().wake_by_ref();
            polled = true;
            Poll::Pending
        }
    });
    let task = spawn_local(future);
    assert_eq!(task.await, 42);
}

/// Check that local futures run on the spawning thread, rather than on
/// the thread where they are woken.
#[wasm_bindgen_test(async)]
async fn multithreading_safe() {
    let spawning_thread_id = wasm_thread::current().id();
    let mut polled = false;
    let future = poll_fn(move |cx| {
        // Check that the future is only polled on the same thread
        // on which it was created.
        let current_thread_id = wasm_thread::current().id();
        assert_eq!(spawning_thread_id, current_thread_id);
        // Put the future to sleep and wake it from a web-worker.
        if polled {
            Poll::Ready(42)
        } else {
            let waker = cx.waker().clone();
            polled = true;
            wasm_thread::spawn(move || {
                waker.wake();
            });
            Poll::Pending
        }
    });
    let task = spawn_local(future);
    assert_eq!(task.await, 42);
}

/// Verify that cancelling a spawn_local future cleans up correctly.
#[wasm_bindgen_test(async)]
async fn cancelled_future_cleans_up() {
    struct Guard(Rc<Cell<bool>>);
    impl Drop for Guard {
        fn drop(&mut self) {
            self.0.set(true);
        }
    }

    let dropped = Rc::new(Cell::new(false));

    // spawn a future that pends forever
    let task = spawn_local({
        let dropped = dropped.clone();
        async move {
            let _guard = Guard(dropped);
            core::future::pending::<()>().await;
        }
    });

    // let the future get polled
    flush_event_loop().await;
    assert!(!dropped.get(), "future should still be alive while pending");

    // cancel the future by dropping the task handle
    drop(task);

    // async_task schedules a final cleanup run rather than dropping the future synchronously
    flush_event_loop().await;
    assert!(dropped.get(), "future should be dropped after cancellation");
}
