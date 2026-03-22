use futures_channel::oneshot;
use js_sys::Promise;
use wasm_bindgen::JsValue;
use wasm_bindgen::closure::Closure;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;

use web_task::spawn_local;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn spawn_local_runs() {
    let task = spawn_local(async { 42 });
    assert_eq!(task.await, 42);
}

#[wasm_bindgen_test]
async fn spawn_local_nested() {
    let (ta, mut ra) = oneshot::channel::<u32>();
    let (ts, rs) = oneshot::channel::<u32>();
    // The order in which the various promises and tasks run is important!
    // We want, on different ticks each, the following things to happen
    // 1. A promise resolves, off of which we can spawn our inbetween assertion
    // 2. The outer task runs, spawns in the inner task, and the inbetween promise, then yields
    // 3. The inbetween promise runs and asserts that the inner task hasn't run
    // 4. The inner task runs
    // This depends crucially on two facts:
    // - JsFuture schedules on ticks independently from tasks
    // - The order of ticks is the same as the code flow
    let promise = Promise::resolve(&JsValue::null());

    let task = spawn_local(async move {
        // Create a closure that runs in between the two ticks and
        // assert that the inner task hasn't run yet
        let inbetween = Closure::wrap(Box::new(move |_| {
            assert_eq!(
                ra.try_recv().unwrap(),
                None,
                "Nested task should not have run yet"
            );
        }) as Box<dyn FnMut(JsValue)>);
        let inbetween = promise.then(&inbetween);
        spawn_local(async {
            ta.send(0xdead).unwrap();
            ts.send(0xbeaf).unwrap();
        })
        .detach();
        JsFuture::from(inbetween).await.unwrap();
        assert_eq!(
            rs.await.unwrap(),
            0xbeaf,
            "Nested task should run eventually"
        );
        42
    });

    assert_eq!(task.await, 42);
}

#[wasm_bindgen_test]
async fn spawn_local_err_no_exception() {
    spawn_local(async {}).detach();
    let task = spawn_local(async { 42 });
    let val = task.await;
    assert_eq!(val, 42);
}
