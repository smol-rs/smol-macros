use event_listener::Event;
use std::sync::atomic::{AtomicBool, Ordering};

/// Wait for the executor to stop.
pub(crate) struct WaitForStop {
    /// Whether or not we need to stop.
    stopped: AtomicBool,

    /// Wait for the stop.
    events: Event,
}

impl WaitForStop {
    /// Create a new wait for stop.
    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            stopped: AtomicBool::new(false),
            events: Event::new(),
        }
    }

    /// Wait for the event to stop.
    #[inline]
    pub(crate) async fn wait(&self) {
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
    pub(crate) fn stop(&self) {
        self.stopped.store(true, Ordering::SeqCst);
        self.events.notify_additional(usize::MAX);
    }
}
