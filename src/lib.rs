use async_task::{Runnable, Task};
use queue::Queue;

extern crate alloc;

mod queue;

/// Spawns a task that is executed by the browser event loop.
pub fn spawn_local<F>(future: F) -> Task<F::Output>
where
    F: Future + 'static,
    F::Output: 'static,
{
    spawn_with(future, Queue::schedule)
}

fn spawn_with<F, S>(future: F, schedule: S) -> Task<F::Output>
where
    F: Future + 'static,
    F::Output: 'static,
    S: Fn(Runnable) + Send + Sync + 'static,
{
    // SAFETY: of the four `spawn_unchecked` requirements:
    //
    // + `future` is not `Send`, so we must show the runnable is only used and
    //    dropped on the same thread. This crate assumes single-threaded
    //    operation, so we assert this is the case.
    //
    // + `future` is `'static`.
    //
    // + `schedule` is `Send` and `Sync`.
    //
    // + `schedule` is `'static`.
    //
    let (runnable, task) = unsafe { async_task::spawn_unchecked(future, &schedule) };
    schedule(runnable);
    task
}
