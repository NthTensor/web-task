#![cfg(target_feature = "atomics")]

use core::future::poll_fn;
use core::task::Poll;

use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

use web_task::spawn_local;

wasm_bindgen_test_configure!(run_in_browser);

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

/// Checkl that local futures run on the spawning thread, rather than on
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
