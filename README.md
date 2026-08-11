# Syncify

Write a module once, get a synchronous copy generated.

`#[syncify::syncify(name)]` on an inline module leaves the module as-is and
generates an additional sibling module with the given name, containing a
synchronous version of the same items: all `async` function modifiers and all
`.await` suffixes are stripped.

Example:

```rust
#[syncify::syncify(greet_sync)]
mod greet {
    pub async fn do_greet(name: &str) -> usize {
        speak(name).await;
        name.len()
    }
}
```

The attributes are invoked with a fully qualified path so they resolve in any
module. When the annotated module lives in the same module as a `use` import,
the bare name works as well:

```rust
use syncify::syncify;

#[syncify(greet_sync)]
mod greet {
    pub async fn do_greet(name: &str) -> usize {
        speak(name).await;
        name.len()
    }
}
```

`use` items can be replaced in the synchronous copy with
`#[syncify::syncify_replace(...)]`:

```rust
#[syncify::syncify(greet_sync)]
mod greet {
    #[syncify::syncify_replace(crate::speaking_sync::speak)] // A sync function for speaking.
    use crate::speaking::speak; // An async function for speaking.

    pub async fn do_greet(name: &str) -> usize {
        speak(name).await;
        name.len()
    }
}
```

Items can be routed to one copy or the other:

* `#[syncify::skip]` keeps the item only in the original module and drops it
  from the generated one (for code that must stay `async`).
* `#[syncify::include]` moves the item out of the original module into the
  generated one (for code that only makes sense synchronously).

```rust
#[syncify::syncify(greet_sync)]
mod greet {
    #[syncify::skip]
    pub async fn stay_async() -> usize {
        42
    }

    #[syncify::include]
    pub fn only_sync() -> usize {
        7
    }
}
```

The markers work on items inside impl blocks and traits too, and on `use` items.

## Futures as return types

Functions returning `impl Future` types are supported in both `impl` blocks and traits.
In the synchronous copy, the `Output` type of the `Future` becomes the return type
of the function.

```rust
#[syncify::syncify(task_sync)]
mod task {
    pub trait Task {
        fn run(&self) -> impl std::future::Future<Output = u32> + Send + '_;
    }
}
```

The generated `task_sync` module contains `fn run(&self) -> u32;`.
