//! Subscriptions — external event sources wired into the TEA loop.
//!
//! A [`Sub`] pairs an identity string with a [`Stream`] of messages.
//! The identity enables lifecycle management: the runner can diff
//! subscriptions between updates, starting new ones and cancelling
//! stale ones automatically.
//!
//! [`Stream`]: futures_core::Stream

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
/// # Examples
///
/// ```
/// use osteak::Sub;
///
/// // A subscription from a channel receiver (using futures Stream)
/// fn from_stream(stream: impl futures_core::Stream<Item = String> + Send + Unpin + 'static) -> Sub<String> {
///     Sub::new("messages", stream)
/// }
/// ```
pub struct Sub<Msg> {
    /// Stable identity for diffing subscriptions across updates.
    pub id: &'static str,

    /// The stream that produces messages.
    pub stream: Box<dyn futures_core::Stream<Item = Msg> + Send + Unpin>,
}

impl<Msg> Sub<Msg> {
    /// Create a new subscription with the given identity and stream.
    pub fn new<S>(id: &'static str, stream: S) -> Self
    where
        S: futures_core::Stream<Item = Msg> + Send + Unpin + 'static,
    {
        Sub {
            id,
            stream: Box::new(stream),
        }
    }
}

impl<Msg> std::fmt::Debug for Sub<Msg> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sub")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}
