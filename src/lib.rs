//! Macros for using `smol-rs`.

#![forbid(unsafe_code)]

#[doc(inline)]
pub use async_executor::{Executor, LocalExecutor};

#[macro_export]
macro_rules! main {
    (
        $(#[$attr:meta])*
        async fn $name:ident () $(-> $ret:ty)? $bl:block
    ) => {
        $(#[$attr])*
        fn $name () $(-> $ret)? {
            $crate::__private::block_on(async {
                $bl
            })
        }
    };

    (
        #[local]
        $(#[$post_attr:meta])*
        async fn $name:ident ($ex:ident : $exty:ty)
        $(-> $ret:ty)? $bl:block
    ) => {
        $(#[$post_attr])*
        fn $name () $(-> $ret)? {
            // Eat the type name.
            fn __eat_the_type_name(_: $exty) {}

            let ex = $crate::__private::Rc::new($crate::LocalExecutor::new());
            let ex = &ex;
            $crate::__private::block_on(ex.run(async move {
                let $ex = ex;
                $bl
            }))
        }
    };

    (
        $(#[$attr:meta])*
        async fn $name:ident ($ex:ident : $exty:ty)
        $(-> $ret:ty)? $bl:block
    ) => {
        $(#[$attr])*
        fn $name () $(-> $ret)? {
            // Eat the type name.
            fn __eat_the_type_name(_: $exty) {}

            $crate::__private::run_executor(|ex| {
                let ex = &ex;
                $crate::__private::block_on(ex.run(async move {
                    let $ex = ex;
                    $bl
                }))
            })
        }
    };
}

#[macro_export]
macro_rules! test {
    () => {
        
    };
}

#[doc(hidden)]
pub mod __private {
    pub use async_io::block_on;
    pub use std::rc::Rc;

    use crate::Executor;
    use event_listener::{Event, EventListener};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::thread;

    /// Run a closure with the executor.
    #[inline]
    pub fn run_executor<T>(f: impl FnOnce(&Arc<Executor<'static>>) -> T) -> T {
        let ex = Arc::new(Executor::new());
        let stopper = WaitForStop::new();

        // Create a thread for each CPU.
        thread::scope(|scope| {
            let num_threads = thread::available_parallelism().map_or(1, |num| num.get());
            for i in 0..num_threads {
                let ex = &ex;
                let stopper = &stopper;

                thread::Builder::new()
                    .name(format!("smol-macros-{i}"))
                    .spawn_scoped(scope, || {
                        block_on(ex.run(stopper.wait()));
                    })
                    .expect("failed to spawn thread");
            }

            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(&ex)));

            stopper.stop();

            match result {
                Ok(value) => value,
                Err(err) => std::panic::resume_unwind(err),
            }
        })
    }

    /// Wait for the executor to stop.
    struct WaitForStop {
        /// Whether or not we need to stop.
        stopped: AtomicBool,

        /// Wait for the stop.
        events: Event,
    }

    impl WaitForStop {
        /// Create a new wait for stop.
        #[inline]
        fn new() -> Self {
            Self {
                stopped: AtomicBool::new(false),
                events: Event::new(),
            }
        }

        /// Wait for the event to stop.
        #[inline]
        async fn wait(&self) {
            let listener = EventListener::new(&self.events);
            futures_lite::pin!(listener);

            loop {
                if self.stopped.load(Ordering::Acquire) {
                    return;
                }

                if listener.is_listening() {
                    listener.as_mut().await;
                } else {
                    listener.as_mut().listen();
                }
            }
        }

        /// Stop the waiter.
        #[inline]
        fn stop(&self) {
            self.stopped.store(true, Ordering::SeqCst);
            self.events.notify_additional(std::usize::MAX);
        }
    }
}
