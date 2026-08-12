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
