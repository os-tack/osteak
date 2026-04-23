//! Subscriptions — external event sources wired into the TEA loop.
//!
//! A [`Sub`] pairs an identity string with a [`Stream`] of messages,
//! **or** with a factory that constructs the stream on demand.
//! The identity enables lifecycle management: the runner can diff
//! subscriptions between updates, starting new ones and cancelling
//! stale ones automatically.
//!
//! [`Stream`]: futures_core::Stream

use std::pin::Pin;

/// Boxed pinned stream used inside the [`Sub`] internal representation.
type BoxStream<Msg> = Pin<Box<dyn futures_core::Stream<Item = Msg> + Send>>;

/// Internal kind: eagerly-owned stream vs. lazy factory.
enum SubKind<Msg> {
    /// Stream already constructed; ready to poll.
    Eager(BoxStream<Msg>),
    /// Factory invoked at spawn-time by the runner. Consumed once.
    Lazy(Box<dyn FnOnce() -> BoxStream<Msg> + Send>),
}

/// A subscription to an external event source.
///
/// Subscriptions produce messages from outside the normal update cycle:
/// timers, file watchers, channels, WebSocket connections, etc.
///
/// The `id` field is a stable identity for lifecycle management. The
/// optional [`runner`](crate::runner) uses it to diff subscriptions:
/// - Same `id` across updates: keep the existing stream running.
/// - New `id`: start the stream and forward its items as messages.
/// - Missing `id`: cancel the stream.
///
/// If you manage your own event loop, you handle subscription lifecycle
/// yourself — the `id` is just a convenience.
///
/// # Construction
///
/// - [`Sub::new`] — eagerly-owned stream. Stream construction must be
///   **side-effect-free**: `subscriptions()` is called on every update,
///   and a `Sub` whose `id` is already active is dropped without ever
///   being polled. If construction acquires a resource (a channel, a
///   thread registration, a global singleton) you will leak/corrupt
///   that resource on every update iteration.
/// - [`Sub::lazy`] — factory invoked **only at spawn-time**. Use this
///   whenever construction is non-trivial or has side effects.
///
/// # Examples
///
/// ```
/// use osteak::Sub;
///
/// // A subscription from a pre-built, cheap-to-construct stream
/// fn from_stream(stream: impl futures_core::Stream<Item = String> + Send + 'static) -> Sub<String> {
///     Sub::new("messages", stream)
/// }
/// ```
pub struct Sub<Msg> {
    /// Stable identity for diffing subscriptions across updates.
    pub id: &'static str,

    kind: SubKind<Msg>,
}

impl<Msg> Sub<Msg> {
    /// Create a new subscription from an already-constructed stream.
    ///
    /// # Invariant
    ///
    /// **The stream must be side-effect-free on construction.**
    ///
    /// `subscriptions()` is called by the runner on every update. A
    /// `Sub` whose id is already active is **dropped** without ever
    /// being polled. If your stream's constructor registers with a
    /// global singleton (e.g. `crossterm::event::EventStream::new()`
    /// which registers a waker with `InternalEventReader`), acquires
    /// a channel, spawns a thread, or otherwise acquires a resource,
    /// that resource leaks (or corrupts shared state) on every
    /// `subscriptions()` call whose id is already active.
    ///
    /// Use [`Sub::lazy`] for factories that acquire resources
    /// (channels, threads, external registrations): the factory is
    /// invoked **only** when the runner actually spawns the stream,
    /// never on drop.
    pub fn new<S>(id: &'static str, stream: S) -> Self
    where
        S: futures_core::Stream<Item = Msg> + Send + 'static,
    {
        Sub {
            id,
            kind: SubKind::Eager(Box::pin(stream)),
        }
    }

    /// Create a new lazy subscription: the `factory` is invoked only
    /// when the runner actually spawns the stream.
    ///
    /// `subscriptions()` is called on every update. The runner
    /// deduplicates by `id` and drops any `Sub` whose id is already
    /// active — the factory attached to a dropped `Sub::lazy` is
    /// **never called**, making this the correct choice for any
    /// construction that has side effects (registering a waker,
    /// allocating a channel, spawning a blocking thread, etc.).
    ///
    /// The factory is `FnOnce + Send + 'static`: it runs at most
    /// once, on whatever tokio worker thread the spawn lands on.
    pub fn lazy<F, S>(id: &'static str, factory: F) -> Self
    where
        F: FnOnce() -> S + Send + 'static,
        S: futures_core::Stream<Item = Msg> + Send + 'static,
    {
        Sub {
            id,
            kind: SubKind::Lazy(Box::new(move || Box::pin(factory()))),
        }
    }

    /// Consume this subscription and return the underlying stream.
    ///
    /// For an `Eager` sub this unwraps the already-constructed
    /// stream. For a `Lazy` sub this invokes the factory **once**
    /// and returns the resulting stream.
    ///
    /// Called by the runner at spawn-time. User code rarely needs
    /// to call this directly.
    pub fn into_stream(self) -> BoxStream<Msg> {
        match self.kind {
            SubKind::Eager(s) => s,
            SubKind::Lazy(factory) => factory(),
        }
    }
}

impl<Msg> std::fmt::Debug for Sub<Msg> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = match &self.kind {
            SubKind::Eager(_) => "Eager",
            SubKind::Lazy(_) => "Lazy",
        };
        f.debug_struct("Sub")
            .field("id", &self.id)
            .field("kind", &kind)
            .finish_non_exhaustive()
    }
}
