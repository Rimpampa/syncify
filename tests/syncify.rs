use futures::executor::block_on;
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

pub mod helper {
    pub async fn value() -> i32 {
        1
    }
}

pub mod helper_sync {
    pub fn value() -> i32 {
        2
    }
}

#[syncify(replace_sync)]
mod replace {
    #[syncify::syncify_replace(crate::helper_sync::value)]
    use crate::helper::value;

    pub async fn get() -> i32 {
        value().await
    }
}

#[test]
fn replace_use_item() {
    assert_eq!(replace_sync::get(), 2);
    assert_eq!(block_on(replace::get()), 1);
}
