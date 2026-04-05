//! Actions returned from [`Tea::update`](crate::Tea::update) to describe side effects.
//!
//! An [`Action`] tells the caller what to do after processing a message.
//! osteak does not execute actions itself — your event loop (or the optional
//! [`runner`](crate::runner)) decides how to spawn tasks and handle quit.

use std::future::Future;
use std::pin::Pin;

/// A side effect to perform after an update.
///
/// `Action` describes *what* to do, not *how*. The caller decides the
/// execution strategy: `tokio::spawn`, `spawn_blocking`, a thread pool,
/// or something else entirely.
///
/// # Examples
///
/// ```
/// use osteak::Action;
///
/// // No side effect
/// let noop: Action<String> = Action::None;
///
/// // An async task that resolves to a message
/// let fetch = Action::task(async { "data loaded".to_string() });
///
/// // Multiple actions
/// let both = Action::Batch(vec![noop, fetch]);
///
/// // Quit the application
/// let quit: Action<String> = Action::Quit;
/// ```
pub enum Action<Msg> {
    /// No side effect.
    None,

    /// Spawn an async task. The future resolves to a message that gets
    /// fed back into [`Tea::update`](crate::Tea::update).
    ///
    /// The caller decides *how* to spawn this (e.g., `tokio::spawn`).
    /// osteak just tells you *what* to spawn.
    Task(Pin<Box<dyn Future<Output = Msg> + Send + 'static>>),

    /// Multiple actions to perform.
    Batch(Vec<Action<Msg>>),

    /// Quit the application.
    Quit,
}

impl<Msg> Action<Msg> {
    /// Create a [`Task`](Action::Task) from any future.
    ///
    /// This is a convenience to avoid writing
    /// `Action::Task(Box::pin(async { ... }))`.
    ///
    /// # Examples
    ///
    /// ```
    /// use osteak::Action;
    ///
    /// let action = Action::task(async { 42 });
    /// ```
    pub fn task<F>(future: F) -> Self
    where
        F: Future<Output = Msg> + Send + 'static,
    {
        Action::Task(Box::pin(future))
    }

    /// Transform the message type of this action.
    ///
    /// This is essential for composing nested models: a child's
    /// `Action<ChildMsg>` becomes `Action<ParentMsg>` via a wrapping
    /// function (typically an enum variant constructor).
    ///
    /// # Examples
    ///
    /// ```
    /// use osteak::Action;
    ///
    /// enum ParentMsg { Child(i32) }
    ///
    /// let child_action: Action<i32> = Action::task(async { 42 });
    /// let parent_action: Action<ParentMsg> = child_action.map(ParentMsg::Child);
    /// ```
    pub fn map<N, F>(self, f: F) -> Action<N>
    where
        Msg: Send + 'static,
        N: Send + 'static,
        F: FnOnce(Msg) -> N + Send + 'static + Clone,
    {
        match self {
            Action::None => Action::None,
            Action::Quit => Action::Quit,
            Action::Task(fut) => Action::Task(Box::pin(async move { f(fut.await) })),
            Action::Batch(actions) => {
                Action::Batch(actions.into_iter().map(|a| a.map(f.clone())).collect())
            }
        }
    }
}

impl<Msg> std::fmt::Debug for Action<Msg> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::None => write!(f, "Action::None"),
            Action::Task(_) => write!(f, "Action::Task(...)"),
            Action::Batch(v) => write!(f, "Action::Batch(len={})", v.len()),
            Action::Quit => write!(f, "Action::Quit"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_constructor() {
        let action = Action::task(async { 42 });
        assert!(matches!(action, Action::Task(_)));
    }

    #[test]
    fn map_none() {
        let action: Action<i32> = Action::None;
        let mapped: Action<String> = action.map(|n| n.to_string());
        assert!(matches!(mapped, Action::None));
    }

    #[test]
    fn map_quit() {
        let action: Action<i32> = Action::Quit;
        let mapped: Action<String> = action.map(|n| n.to_string());
        assert!(matches!(mapped, Action::Quit));
    }

    #[test]
    fn map_batch() {
        let action: Action<i32> = Action::Batch(vec![Action::None, Action::Quit]);
        let mapped: Action<String> = action.map(|n| n.to_string());
        match mapped {
            Action::Batch(v) => {
                assert_eq!(v.len(), 2);
                assert!(matches!(v[0], Action::None));
                assert!(matches!(v[1], Action::Quit));
            }
            _ => panic!("expected Batch"),
        }
    }

    #[test]
    fn debug_formatting() {
        let none: Action<i32> = Action::None;
        assert_eq!(format!("{none:?}"), "Action::None");

        let quit: Action<i32> = Action::Quit;
        assert_eq!(format!("{quit:?}"), "Action::Quit");

        let task = Action::task(async { 1 });
        assert_eq!(format!("{task:?}"), "Action::Task(...)");

        let batch: Action<i32> = Action::Batch(vec![Action::None]);
        assert_eq!(format!("{batch:?}"), "Action::Batch(len=1)");
    }
}
