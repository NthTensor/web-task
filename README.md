This crate is a modern alternative to `web-bindgen-futures` built around `async-task`.

# Pros & Cons

**Pros:**
+ Provides both `spawn` (for `Send` futures) and `spawn_local` (for `!Send` futures).
+ `spawn` and `spawn_local` return `Task<T>` futures, which resolve to return values.
+ It's possible to cancel futures using task handles.
+ Non-send futures (which are common on web) can have send task-handles.

**Cons:**
+ Some older browser versions are not supported.

# Runtime Support

Enabling the `+atomics` nightly target feature automatically switches the
crate to a multithreaded runtime, which may have different performance characteristics.
