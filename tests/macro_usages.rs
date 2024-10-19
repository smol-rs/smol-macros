//! Testing the test macros.

use async_lock::Barrier;
use futures_lite::prelude::*;
use macro_rules_attribute::apply;
use smol_macros::{test, Executor, LocalExecutor};

use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

test! {
    async fn basic_test() {
        println!("test 1");
    }
}

#[apply(test!)]
async fn with_attribute() {
    println!("test 2");
}

#[apply(test!)]
async fn with_executor(ex: &Executor<'static>) {
    let barrier = Arc::new(Barrier::new(2));
    ex.spawn({
        let barrier = barrier.clone();
        async move {
            barrier.wait().await;
        }
    })
    .detach();
    barrier
        .wait()
        .or(async {
            async_io::Timer::after(Duration::from_secs(5)).await;
            panic!("timed out")
        })
        .await;
}

#[apply(test!)]
async fn with_executor_arc(ex: Arc<Executor<'static>>) {
    let barrier = Arc::new(Barrier::new(2));
    ex.spawn({
        let barrier = barrier.clone();
        async move {
            barrier.wait().await;
        }
    })
    .detach();
    barrier
        .wait()
        .or(async {
            async_io::Timer::after(Duration::from_secs(5)).await;
            panic!("timed out")
        })
        .await;
}

#[apply(test!)]
async fn with_executor_arcref(ex: &Arc<Executor<'static>>) {
    let barrier = Arc::new(Barrier::new(2));
    ex.spawn({
        let barrier = barrier.clone();
        async move {
            barrier.wait().await;
        }
    })
    .detach();
    barrier
        .wait()
        .or(async {
            async_io::Timer::after(Duration::from_secs(5)).await;
            panic!("timed out")
        })
        .await;
}

#[apply(test!)]
async fn with_local(ex: &LocalExecutor<'_>) {
    let barrier = Rc::new(unsend::lock::Barrier::new(2));
    ex.spawn({
        let barrier = barrier.clone();
        async move {
            barrier.wait().await;
        }
    })
    .detach();
    barrier
        .wait()
        .or(async {
            async_io::Timer::after(Duration::from_secs(5)).await;
            panic!("timed out")
        })
        .await;
}

#[apply(test!)]
async fn with_local_rc(ex: Rc<LocalExecutor<'_>>) {
    let barrier = Rc::new(unsend::lock::Barrier::new(2));
    ex.spawn({
        let barrier = barrier.clone();
        async move {
            barrier.wait().await;
        }
    })
    .detach();
    barrier
        .wait()
        .or(async {
            async_io::Timer::after(Duration::from_secs(5)).await;
            panic!("timed out")
        })
        .await;
}

#[apply(test!)]
async fn with_local_rcref(ex: &Rc<LocalExecutor<'_>>) {
    let barrier = Rc::new(unsend::lock::Barrier::new(2));
    ex.spawn({
        let barrier = barrier.clone();
        async move {
            barrier.wait().await;
        }
    })
    .detach();
    barrier
        .wait()
        .or(async {
            async_io::Timer::after(Duration::from_secs(5)).await;
            panic!("timed out")
        })
        .await;
}

#[apply(test!)]
async fn it_works(_ex: &Executor<'_>) -> Result<(), Box<dyn std::error::Error>> {
    let _ = u32::try_from(20usize)?;
    Ok(())
}
