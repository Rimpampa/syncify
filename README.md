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
