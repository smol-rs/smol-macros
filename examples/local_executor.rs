//! Set up a thread executor that is local.

use macro_rules_attribute::apply;
use smol_macros::{main, LocalExecutor};
use std::time::Duration;

#[apply(main!)]
#[local]
async fn main(ex: &LocalExecutor<'_>) {
    let mut tasks = vec![];
    for i in 0..16 {
        let task = ex.spawn(async move {
            println!("Task number {i}");
        });

        tasks.push(task);
    }

    async_io::Timer::after(Duration::from_secs(1)).await;

    // Wait for tasks to complete.
    for task in tasks {
        task.await;
    }
}
