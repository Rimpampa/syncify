mod common;

use common::block_on;
use syncify::syncify;

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

#[syncify(block_multistmt_sync)]
mod block_multistmt {
    pub async fn compute() -> u32 {
        let x = async {
            let a = 2;
            let b = 3;
            a + b
        }
        .await;
        x + 1
    }
}

#[test]
fn async_block_multistatement() {
    assert_eq!(block_multistmt_sync::compute(), 6);
    assert_eq!(block_on(block_multistmt::compute()), 6);
}

#[syncify(closure_move_sync)]
mod closure_move {
    pub async fn compute() -> i32 {
        let v = [1, 2, 3];
        let sum = async move || v.iter().sum::<i32>();
        sum().await
    }
}

#[test]
fn async_move_closures() {
    assert_eq!(closure_move_sync::compute(), 6);
    assert_eq!(block_on(closure_move::compute()), 6);
}

#[syncify(closure_await_sync)]
mod closure_await {
    async fn helper() -> u32 {
        10
    }

    pub async fn compute() -> u32 {
        let f = async || helper().await + 1;
        f().await
    }
}

#[test]
fn async_closure_with_await() {
    assert_eq!(closure_await_sync::compute(), 11);
    assert_eq!(block_on(closure_await::compute()), 11);
}

#[syncify(chained_await_sync)]
mod chained_await {
    #[allow(clippy::async_yields_async)]
    pub async fn compute() -> i32 {
        async { async { 5 } }.await.await
    }
}

#[test]
fn chained_await() {
    assert_eq!(chained_await_sync::compute(), 5);
    assert_eq!(block_on(chained_await::compute()), 5);
}

#[syncify(nested_blocks_sync)]
mod nested_blocks {
    pub async fn compute() -> i32 {
        let x = async {
            let y = async { 5 }.await;
            y + 1
        }
        .await;
        x + 1
    }
}

#[test]
fn nested_async_blocks() {
    assert_eq!(nested_blocks_sync::compute(), 7);
    assert_eq!(block_on(nested_blocks::compute()), 7);
}
