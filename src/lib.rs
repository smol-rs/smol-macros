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
        $(#[$post_attr:meta])*
        async fn $name:ident ($ex:ident : & $exty:ty)
        $(-> $ret:ty)? $bl:block
    ) => {
        $(#[$post_attr])*
        fn $name () $(-> $ret)? {
            <$exty as $crate::__private::MainExecutor>::with_main(|ex| {
                $crate::__private::block_on(ex.run(async move {
                    let $ex = ex;
                    $bl
                }))
            })
        }
    };

    (
        $(#[$post_attr:meta])*
        async fn $name:ident ($ex:ident : $exty:ty)
        $(-> $ret:ty)? $bl:block
    ) => {
        $crate::main! {
            $(#[$post_attr])*
            async fn $name(ex: &$exty) $(-> $ret)? {
                let $ex = ex.clone();
                $bl
            }
        }
    }
}

#[macro_export]
macro_rules! test {
    () => {};
}

#[doc(hidden)]
pub mod __private {
    pub use async_io::block_on;
    pub use std::rc::Rc;

    use crate::{Executor, LocalExecutor};
    use event_listener::{Event, EventListener};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::thread;

    /// Something that can be set up as an executor.
    #[doc(hidden)]
    pub trait MainExecutor: Sized {
        /// Create this type and pass it into `main`.
        fn with_main<T, F: FnOnce(&Self) -> T>(f: F) -> T;
    }

    impl MainExecutor for Arc<Executor<'_>> {
        #[inline]
        fn with_main<T, F: FnOnce(&Self) -> T>(f: F) -> T {
            let ex = Arc::new(Executor::new());
            with_thread_pool(&ex, || f(&ex))
        }
    }

    impl MainExecutor for Executor<'_> {
        #[inline]
        fn with_main<T, F: FnOnce(&Self) -> T>(f: F) -> T {
            let ex = Executor::new();
            with_thread_pool(&ex, || f(&ex))
        }
    }

    impl MainExecutor for Rc<LocalExecutor<'_>> {
        #[inline]
        fn with_main<T, F: FnOnce(&Self) -> T>(f: F) -> T {
            f(&Rc::new(LocalExecutor::new()))
        }
    }

    impl MainExecutor for LocalExecutor<'_> {
        fn with_main<T, F: FnOnce(&Self) -> T>(f: F) -> T {
            f(&LocalExecutor::new())
        }
    }

    /// Run a function that takes an `Executor` inside of a thread pool.
    #[inline]
    fn with_thread_pool<T>(ex: &Executor<'_>, f: impl FnOnce() -> T) -> T {
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

            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));

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
