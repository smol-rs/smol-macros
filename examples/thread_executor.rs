//! Set up a thread executor.

use smol_macros::{main, Executor};
use std::time::Duration;

main! {
    async fn main(ex: &Executor<'_>) {
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
}
