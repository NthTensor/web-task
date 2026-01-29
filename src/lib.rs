//! This crate is a modern alternative to `web-bindgen-futures` built around `async-task`.
//!
//! # Pros & Cons
//!
//! **Pros:**
//! + Provides both [`spawn`] (for `Send` futures) and [`spawn_local`] (for `!Send` futures).
//! + [`spawn`] and [`spawn_local`] return [`Task<T>`](async_task::Task) futures, which resolve to return values.
//! + It's possible to cancel futures using task handles.
//! + Non-send futures (which are common on web) can have send task-handles.
//!
//! **Cons:**
//! + Some older browser versions are not supported.
//!
//! # Runtime Support
//!
//! Enabling the `+atomics` nightly target feature automatically switches the
//! crate to a multithreaded runtime, which may have different performance charictaristics.

#![no_std]
#![cfg_attr(
    target_feature = "atomics",
    feature(thread_local, stdarch_wasm_atomic_wait)
)]
#![deny(missing_docs)]

extern crate alloc;

use async_task::Task;

mod queue;

mod runtime {
    use cfg_if::cfg_if;

    cfg_if! {
        if #[cfg(target_feature = "atomics")] {
            mod multithread;
            pub(crate) use multithread::*;

        } else {
            mod singlethread;
            pub(crate) use singlethread::*;
         }
    }
}

/// Spawns a [`Future<Output = T>`](core::future::Future) that can execute on
/// any thread; returns a [`Task`].
///
/// The future will be polled to completion in the background. Awaiting the
/// returned task has no effect when the future is polled. Dropping the task
/// will cancel the future, unless you call [`Task::detach()`] first.
#[inline]
pub fn spawn<F>(future: F) -> Task<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    runtime::Job::spawn(future)
}

/// Spawns a [`Future<Output = T>`](core::future::Future) that executes on the
/// current thread; returns a [`Task`].
///
/// The future will be polled to completion in the background. Awaiting the
/// returned task has no effect when the future is polled. Dropping the task
/// will cancel the future, unless you call [`Task::detach()`] first.
#[inline]
pub fn spawn_local<F>(future: F) -> Task<F::Output>
where
    F: Future + 'static,
    F::Output: 'static,
{
    runtime::Job::spawn_local(future)
}
