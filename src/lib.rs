//! Macros for using `smol-rs`.
//!
//! One of the advantages of [`smol`] is that it lets you set up your own executor, optimized for
//! your own use cases. However, quick scaffolding is important for many organizational use cases.
//! Especially when sane defaults are appreciated, setting up your own executor is a waste of
//! time.
//!
//! This crate provides macros for setting up an efficient [`smol`] runtime quickly and
//! effectively. It provides sane defaults that are useful for most applications.
//!
//! ## Simple Executor
//!
//! Just have an `async` main function, using the [`main`] macro.
//!
//!
//! ```
//! use smol_macros::main;
//!
//! main! {
//!     async fn main() {
//!         println!("Hello, world!");
//!     }
//! }
//! ```
//!
//! This crate uses declarative macros rather than procedural macros, in order to avoid needing
//! to use heavy macro dependencies. If you want to use the proc macro syntax, you can use the
//! [`macro_rules_attribute::apply`] function to emulate it.
//!
//! The following is equivalent to the previous example.
//!
//! ```
//! use macro_rules_attribute::apply;
//! use smol_macros::main;
//!
//! #[apply(main!)]
//! async fn main() {
//!     println!("Hello, world!");
//! }
//! ```
//!
//! ## Task-Based Executor
//!
//! This crate re-exports [`smol::Executor`]. If that is used as the first parameter in a
//! function in [`main`], it will automatically create the executor.
//!
//! ```
//! use macro_rules_attribute::apply;
//! use smol_macros::{main, Executor};
//!
//! #[apply(main!)]
//! async fn main(ex: &Executor<'_>) {
//!     ex.spawn(async { println!("Hello world!"); }).await;
//! }
//! ```
//!
//! If the thread-safe [`smol::Executor`] is used here, a thread pool will be spawned to run
//! the executor on multiple threads. For the thread-unsafe [`smol::LocalExecutor`], no threads
//! will be spawned.
//!
//! See documentation for the [`main`] function for more details.
//!
//! ## Tests
//!
//! Use the [`test`] macro to set up test cases that run self-contained executors.
//!
//! ```
//! use macro_rules_attribute::apply;
//! use smol_macros::{test, Executor};
//!
//! #[apply(test!)]
//! async fn do_test(ex: &Executor<'_>) {
//!     ex.spawn(async {
//!         assert_eq!(1 + 1, 2);
//!     }).await;
//! }
//! ```
//!
//! [`smol`]: https://crates.io/crates/smol
//! [`smol::Executor`]: https://docs.rs/smol/latest/smol/struct.Executor.html
//! [`smol::LocalExecutor`]: https://docs.rs/smol/latest/smol/struct.LocalExecutor.html
//! [`macro_rules_attribute::apply`]: https://docs.rs/macro_rules_attribute/latest/macro_rules_attribute/attr.apply.html

#![forbid(unsafe_code)]

#[doc(no_inline)]
pub use async_executor::{Executor, LocalExecutor};

/// Turn a main function into one that runs inside of a self-contained executor.
///
/// The function created by this macro spawns an executor, spawns threads to run that executor
/// on (if applicable), and then blocks the current thread on the future.
///
/// ## Examples
///
/// Like [`tokio::main`], this function is not limited to wrapping the program's entry point.
/// In a mostly synchronous program, it can wrap a self-contained `async` function in its
/// own executor.
///
/// ```
/// use macro_rules_attribute::apply;
/// use smol_macros::{main, Executor};
///
/// fn do_something_sync() -> u32 {
///     1 + 1
/// }
///
/// #[apply(main!)]
/// async fn do_something_async(ex: &Executor<'_>) -> u32 {
///     ex.spawn(async { 1 + 1 }).await
/// }
///
/// fn main() {
///     let x = do_something_sync();
///     let y = do_something_async();
///     assert_eq!(x + y, 4);
/// }
/// ```
///
/// The first parameter to the `main` function can be an executor. It can be one of the following:
///
/// - Nothing.
/// - `&`[`Executor`]
/// - `&`[`LocalExecutor`]
/// - `Arc<`[`Executor`]`>`
/// - `Rc<`[`LocalExecutor`]`>`
///
/// [`tokio::main`]: https://docs.rs/tokio/latest/tokio/attr.main.html
/// [`Executor`]: https://docs.rs/smol/latest/smol/struct.Executor.html
/// [`LocalExecutor`]: https://docs.rs/smol/latest/smol/struct.LocalExecutor.html
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

/// Wrap a test in an asynchronous executor.
///
/// This is equivalent to the [`main`] macro, but adds the `#[test]` attribute.
///
/// ## Examples
///
/// ```
/// use macro_rules_attribute::apply;
/// use smol_macros::test;
///
/// #[apply(test!)]
/// async fn do_test() {
///     assert_eq!(1 + 1, 2);
/// }
/// ```
#[macro_export]
macro_rules! test {
    // Special case to get around bug in macro engine.
    (
        $(#[$post_attr:meta])*
        async fn $name:ident ($exname:ident : & $exty:ty)
        $(-> $ret:ty)? $bl:block
    ) => {
        $crate::main! {
            $(#[$post_attr])*
            #[core::prelude::v1::test]
            async fn $name($exname: &$exty) $(-> $ret)? $bl
        }
    };

    (
        $(#[$post_attr:meta])*
        async fn $name:ident ($($pname:ident : $pty:ty),* $(,)?)
        $(-> $ret:ty)? $bl:block
    ) => {
        $crate::main! {
            $(#[$post_attr])*
            #[core::prelude::v1::test]
            async fn $name($($pname: $pty),*) $(-> $ret)? $bl
        }
    };
}

#[doc(hidden)]
pub mod __private {
    pub use async_io::block_on;
    pub use std::rc::Rc;

    use crate::{Executor, LocalExecutor};
    use event_listener::Event;
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
            loop {
                if self.stopped.load(Ordering::Relaxed) {
                    return;
                }

                event_listener::listener!(&self.events => listener);

                if self.stopped.load(Ordering::Acquire) {
                    return;
                }

                listener.await;
            }
        }

        /// Stop the waiter.
        #[inline]
        fn stop(&self) {
            self.stopped.store(true, Ordering::SeqCst);
            self.events.notify_additional(usize::MAX);
        }
    }
}
