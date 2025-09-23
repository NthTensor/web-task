# web-task

This is fork of `wasm-bindgen-futures` that uses `async-task` directly. It provides a `spawn_local` function that returns a `Task`.

## Correct Use

This crate only works on the web, and does not yet support web workers or web multi-threading.
