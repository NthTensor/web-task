use alloc::collections::VecDeque;
use alloc::rc::Rc;
use async_task::Runnable;
use core::cell::{Cell, RefCell};
use core::mem;
use js_sys::Promise;
use std::ops::DerefMut;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    fn queueMicrotask(closure: &Closure<dyn FnMut(JsValue)>);

    type Global;

    #[wasm_bindgen(method, getter, js_name = queueMicrotask)]
    fn hasQueueMicrotask(this: &Global) -> JsValue;
}

struct QueueState {
    // The queue of Tasks which are to be run in order. In practice this is all the
    // synchronous work of futures, and each `Task` represents calling `poll` on
    // a future "at the right time".
    runnables: RefCell<VecDeque<Runnable>>,

    // This flag indicates whether we've scheduled `run_all` to run in the future.
    // This is used to ensure that it's only scheduled once.
    is_scheduled: Cell<bool>,
}

impl QueueState {
    fn run_all(&self) {
        // "consume" the schedule
        let _was_scheduled = self.is_scheduled.replace(false);
        debug_assert!(_was_scheduled);

        // Takes everything already scheduled and runs it. Tasks scheduled after this point will run in the next microtask.
        let runnables = mem::take(self.runnables.borrow_mut().deref_mut());
        for runnable in runnables {
            runnable.run();
        }

        // All of the Tasks have been run, so it's now possible to schedule the
        // next tick again
    }
}

pub(crate) struct Queue {
    state: Rc<QueueState>,
    promise: Promise,
    closure: Closure<dyn FnMut(JsValue)>,
    has_queue_microtask: bool,
}

impl Queue {
    // Schedule a task to run on the next tick.
    pub(crate) fn schedule(runnable: Runnable) {
        Self::with(|queue| {
            queue.state.runnables.borrow_mut().push_back(runnable);
            // Use queueMicrotask to execute as soon as possible. If it does not exist
            // fall back to the promise resolution
            if !queue.state.is_scheduled.replace(true) {
                if queue.has_queue_microtask {
                    queueMicrotask(&queue.closure);
                } else {
                    let _ = queue.promise.then(&queue.closure);
                }
            }
        })
    }
}

impl Queue {
    fn new() -> Self {
        let state = Rc::new(QueueState {
            is_scheduled: Cell::new(false),
            runnables: RefCell::new(VecDeque::new()),
        });

        let has_queue_microtask = js_sys::global()
            .unchecked_into::<Global>()
            .hasQueueMicrotask()
            .is_function();

        Self {
            promise: Promise::resolve(&JsValue::undefined()),

            closure: {
                let state = Rc::clone(&state);

                // This closure will only be called on the next microtask event
                // tick
                Closure::new(move |_| state.run_all())
            },

            state,
            has_queue_microtask,
        }
    }

    fn with<R>(f: impl FnOnce(&Self) -> R) -> R {
        use once_cell::unsync::Lazy;

        struct Wrapper<T>(Lazy<T>);

        unsafe impl<T> Sync for Wrapper<T> {}

        unsafe impl<T> Send for Wrapper<T> {}

        static QUEUE: Wrapper<Queue> = Wrapper(Lazy::new(Queue::new));

        f(&QUEUE.0)
    }
}
