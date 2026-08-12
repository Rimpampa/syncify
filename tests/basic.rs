mod common;

use common::block_on;
use syncify::syncify;

#[syncify(test_sync)]
mod test {
    pub async fn foo() -> i32 {
        bar().await + 1
    }

    pub async fn bar() -> i32 {
        5
    }
}

#[test]
fn basic_sync() {
    assert_eq!(test_sync::foo(), 6);
}

#[test]
fn basic_async() {
    assert_eq!(block_on(test::foo()), 6);
}

#[syncify(nested_mod_sync)]
mod nested_mod {
    pub mod inner {
        pub async fn val() -> u32 {
            helper().await
        }

        async fn helper() -> u32 {
            8
        }
    }

    pub async fn compute() -> u32 {
        inner::val().await + 1
    }
}

#[test]
fn nested_modules() {
    assert_eq!(nested_mod_sync::compute(), 9);
    assert_eq!(block_on(nested_mod::compute()), 9);
}
