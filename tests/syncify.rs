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

#[syncify(only_sync)]
mod only {
    #[syncify::syncify_skip]
    pub async fn async_only_val() -> u32 {
        1
    }

    #[syncify::syncify_include]
    pub fn blocking_only_val() -> u32 {
        2
    }
}

#[test]
fn skip_keeps_item_in_original() {
    assert_eq!(block_on(only::async_only_val()), 1);
}

#[test]
fn include_moves_item_to_sync() {
    assert_eq!(only_sync::blocking_only_val(), 2);
}

#[syncify(marker_impl_sync)]
mod marker_impl {
    pub struct State(pub i32);

    impl State {
        #[syncify::syncify_skip]
        pub async fn async_get(&self) -> i32 {
            self.0
        }

        #[syncify::syncify_include]
        pub fn blocking_get(&self) -> i32 {
            self.0
        }

        pub async fn both_get(&self) -> i32 {
            self.0
        }
    }
}

#[test]
fn impl_marker_routing() {
    let blocking_state = marker_impl_sync::State(3);
    assert_eq!(blocking_state.both_get(), 3);
    assert_eq!(blocking_state.blocking_get(), 3);

    let asynch_state = marker_impl::State(3);
    assert_eq!(block_on(asynch_state.both_get()), 3);
    assert_eq!(block_on(asynch_state.async_get()), 3);
}

#[syncify(blocks_sync)]
mod blocks {
    pub async fn compute() -> u32 {
        let x = async { 5u32 }.await;
        let y = async { 7u32 }.await;
        x + y
    }
}

#[test]
fn async_blocks() {
    assert_eq!(blocks_sync::compute(), 12);
    assert_eq!(block_on(blocks::compute()), 12);
}

#[syncify(closure_sync)]
mod closure {
    pub async fn compute() -> u32 {
        let add = async |x: u32, y: u32| x + y;
        add(5u32, 7u32).await
    }
}

#[test]
fn async_closures() {
    assert_eq!(closure_sync::compute(), 12);
    assert_eq!(block_on(closure::compute()), 12);
}

#[syncify(asyncfn_sync)]
mod asyncfn {
    pub async fn call<F>(f: F) -> u32
    where
        F: AsyncFnOnce() -> u32,
    {
        f().await
    }
}

#[test]
fn asyncfn_to_fn() {
    assert_eq!(asyncfn_sync::call(|| 42u32), 42);
    assert_eq!(block_on(asyncfn::call(async || 42u32)), 42);
}

#[syncify(traits_rpitit_sync)]
mod traits_rpitit {
    pub trait Task {
        // Desugared AFIT form, avoids the `async_fn_in_trait` warning.
        fn run(&self) -> impl std::future::Future<Output = u32> + Send + '_;
    }

    pub struct Job;

    impl Task for Job {
        async fn run(&self) -> u32 {
            42
        }
    }
}

#[test]
fn trait_desugared_form() {
    use traits_rpitit::Task as _;
    use traits_rpitit_sync::Task as _;
    assert_eq!(traits_rpitit_sync::Job.run(), 42);
    assert_eq!(block_on(traits_rpitit::Job.run()), 42);
}
