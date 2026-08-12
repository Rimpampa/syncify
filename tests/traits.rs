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

#[syncify(asyncfn_ref_sync)]
mod asyncfn_ref {
    pub async fn apply<F>(f: F) -> u32
    where
        F: AsyncFn() -> u32,
    {
        f().await + f().await
    }
}

#[test]
fn asyncfn_ref_to_fn() {
    assert_eq!(asyncfn_ref_sync::apply(|| 7u32), 14);
    assert_eq!(block_on(asyncfn_ref::apply(async || 7u32)), 14);
}

#[syncify(asyncfn_mut_sync)]
mod asyncfn_mut {
    pub async fn run<F>(mut f: F) -> u32
    where
        F: AsyncFnMut() -> u32,
    {
        let first = f().await;
        let second = f().await;
        first * 10 + second
    }
}

#[test]
fn asyncfn_mut_to_fnmut() {
    let mut n = 0;
    let mut inc = || {
        n += 1;
        n
    };
    assert_eq!(asyncfn_mut_sync::run(&mut inc), 12);
    assert_eq!(n, 2);

    let mut n = 0;
    let mut inc = async || {
        n += 1;
        n
    };
    assert_eq!(block_on(asyncfn_mut::run(&mut inc)), 12);
    assert_eq!(n, 2);
}

#[syncify(trait_asyncfn_sync)]
mod trait_asyncfn {
    pub trait Runner {
        async fn run<F>(&self, f: F) -> u32
        where
            F: AsyncFnOnce() -> u32;
    }

    pub struct Job;

    impl Runner for Job {
        async fn run<F>(&self, f: F) -> u32
        where
            F: AsyncFnOnce() -> u32,
        {
            f().await
        }
    }
}

#[test]
fn asyncfn_in_trait() {
    use trait_asyncfn::Runner as _;
    use trait_asyncfn_sync::Runner as _;

    assert_eq!(block_on(trait_asyncfn::Job.run(async || 9u32)), 9);
    assert_eq!(trait_asyncfn_sync::Job.run(|| 9u32), 9);
}

#[syncify(future_fn_sync)]
mod future_fn {
    #[allow(clippy::manual_async_fn)]
    pub fn compute() -> impl std::future::Future<Output = u32> {
        async { 21 }
    }
}

#[test]
fn standalone_future_return() {
    assert_eq!(future_fn_sync::compute(), 21);
    assert_eq!(block_on(future_fn::compute()), 21);
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
