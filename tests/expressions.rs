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
