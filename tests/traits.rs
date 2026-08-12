mod common;

use common::block_on;
use syncify::syncify;

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
