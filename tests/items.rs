mod common;

use common::block_on;
use syncify::syncify;

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

#[syncify(trait_markers_sync)]
mod trait_markers {
    pub trait Task {
        #[syncify::syncify_skip]
        async fn async_only(&self) -> u32;

        #[syncify::syncify_include]
        fn sync_only(&self) -> u32;

        async fn both(&self) -> u32;
    }

    pub struct Job;

    impl Task for Job {
        #[syncify::syncify_skip]
        async fn async_only(&self) -> u32 {
            1
        }

        #[syncify::syncify_include]
        fn sync_only(&self) -> u32 {
            2
        }

        async fn both(&self) -> u32 {
            3
        }
    }
}

#[test]
fn trait_marker_routing() {
    use trait_markers::Task as _;
    use trait_markers_sync::Task as _;

    assert_eq!(block_on(trait_markers::Job.async_only()), 1);
    assert_eq!(block_on(trait_markers::Job.both()), 3);
    assert_eq!(trait_markers_sync::Job.both(), 3);
    assert_eq!(trait_markers_sync::Job.sync_only(), 2);
}
