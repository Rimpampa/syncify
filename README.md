# Syncify

[![crates.io](https://img.shields.io/crates/v/syncify.svg)](https://crates.io/crates/syncify)
[![docs.rs](https://docs.rs/syncify/badge.svg)](https://docs.rs/syncify)

Write a module once, get a synchronous copy generated.

`#[syncify::syncify(name)]` on an inline module leaves the module as-is and
generates an additional sibling module with the given name, containing a
synchronous version of the same items.

```rust
use syncify::*;

#[syncify(repo_sync)]
mod repo {
    use super::*;

    // Swaps the async `foo` for a sync counterpart.
    #[syncify_replace(sync_crate::foo)]
    use async_crate::foo;

    // `async fn` becomes `fn`.
    pub async fn bar() {
        // `.await` suffixes are stripped.
        foo().await;
    }

    pub fn baz<T>(
        // `AsyncFn*` becomes `Fn*`.
        f: impl AsyncFn() -> T
    // `impl Future<Output = T>` becomes `T`.
    ) -> impl Future<Output = T> {
        // `async { .. }` becomes `{ .. }`.
        async {
            f().await
        }
    }

    // `syncify_skip`: kept only in the original (async) module.
    #[syncify_skip]
    pub async fn async_only() {}

    // `syncify_include`: kept only in the generated (sync) module.
    #[syncify_include]
    pub fn blocking_only() {}
}
```

Results in the generated `repo_sync` module:

```rust
mod repo {
    use async_crate::foo;

    pub async fn bar() {
        foo().await;
    }

    pub fn baz<T>(
        f: impl AsyncFn() -> T
    ) -> impl Future<Output = T> {
        async {
            f().await
        }
    }

    pub async fn async_only() {}
}

mod repo_sync {
    use sync_crate::foo;

    pub fn bar() {
        foo();
    }

    pub fn baz<T>(f: impl Fn() -> T) -> T {
        f()
    }

    pub fn blocking_only() {}
}
```

See the [documentation](https://docs.rs/syncify) for the full reference.
