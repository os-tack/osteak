//! The return type from [`Tea::update`](crate::Tea::update).
//!
//! A [`Cmd`] bundles an [`Action`] (side effect) with a dirty flag
//! (whether the view needs re-rendering). Convenience constructors
//! make the common cases concise.

use std::future::Future;

use crate::Action;

/// The result of processing a message in [`Tea::update`](crate::Tea::update).
///
/// Contains an [`Action`] describing what side effect to perform (if any)
/// and a `dirty` flag indicating whether the view should be re-rendered.
///
/// # Examples
///
/// ```
/// use osteak::Cmd;
///
/// // State changed, no side effect
/// let cmd: Cmd<()> = Cmd::dirty();
/// assert!(cmd.dirty);
///
/// // Nothing happened
/// let cmd: Cmd<()> = Cmd::none();
/// assert!(!cmd.dirty);
///
/// // Quit the app
/// let cmd: Cmd<()> = Cmd::quit();
/// ```
pub struct Cmd<Msg> {
    /// The side effect to perform after this update.
    pub action: Action<Msg>,

    /// Whether the view needs re-rendering after this update.
    pub dirty: bool,
}

impl<Msg> Cmd<Msg> {
    /// No side effect, no re-render needed.
    pub fn none() -> Self {
        Cmd {
            action: Action::None,
            dirty: false,
        }
    }

    /// No side effect, but the view needs re-rendering.
    ///
    /// Use this when `update` mutated state that affects the view.
    pub fn dirty() -> Self {
        Cmd {
            action: Action::None,
            dirty: true,
        }
    }

    /// Quit the application.
    pub fn quit() -> Self {
        Cmd {
            action: Action::Quit,
            dirty: false,
        }
    }

    /// Spawn an async task and mark the view dirty.
    ///
    /// # Examples
    ///
    /// ```
    /// use osteak::Cmd;
    ///
    /// let cmd = Cmd::task(async { "loaded".to_string() });
    /// assert!(cmd.dirty);
    /// ```
    pub fn task<F>(future: F) -> Self
    where
        F: Future<Output = Msg> + Send + 'static,
    {
        Cmd {
            action: Action::task(future),
            dirty: true,
        }
    }

    /// Perform multiple actions and mark the view dirty.
    pub fn batch(actions: Vec<Action<Msg>>) -> Self {
        Cmd {
            action: Action::Batch(actions),
            dirty: true,
        }
    }

    /// Perform an action with explicit dirty control.
    pub fn with_action(action: Action<Msg>, dirty: bool) -> Self {
        Cmd { action, dirty }
    }

    /// Transform the message type of this command.
    ///
    /// Essential for composing nested models: a child's `Cmd<ChildMsg>`
    /// becomes `Cmd<ParentMsg>` via a wrapping function.
    ///
    /// # Examples
    ///
    /// ```
    /// use osteak::Cmd;
    ///
    /// enum Parent { Child(i32) }
    ///
    /// let child_cmd: Cmd<i32> = Cmd::dirty();
    /// let parent_cmd: Cmd<Parent> = child_cmd.map(Parent::Child);
    /// ```
    pub fn map<N, F>(self, f: F) -> Cmd<N>
    where
        Msg: Send + 'static,
        N: Send + 'static,
        F: FnOnce(Msg) -> N + Send + 'static + Clone,
    {
        Cmd {
            action: self.action.map(f),
            dirty: self.dirty,
        }
    }
}

impl<Msg> std::fmt::Debug for Cmd<Msg> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cmd")
            .field("action", &self.action)
            .field("dirty", &self.dirty)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn none_is_clean() {
        let cmd: Cmd<()> = Cmd::none();
        assert!(!cmd.dirty);
        assert!(matches!(cmd.action, Action::None));
    }

    #[test]
    fn dirty_is_dirty() {
        let cmd: Cmd<()> = Cmd::dirty();
        assert!(cmd.dirty);
        assert!(matches!(cmd.action, Action::None));
    }

    #[test]
    fn quit_action() {
        let cmd: Cmd<()> = Cmd::quit();
        assert!(matches!(cmd.action, Action::Quit));
    }

    #[test]
    fn task_is_dirty() {
        let cmd = Cmd::task(async { 42 });
        assert!(cmd.dirty);
        assert!(matches!(cmd.action, Action::Task(_)));
    }

    #[test]
    fn map_preserves_dirty() {
        let cmd: Cmd<i32> = Cmd::dirty();
        let mapped: Cmd<String> = cmd.map(|n| n.to_string());
        assert!(mapped.dirty);
    }

    #[test]
    fn map_preserves_clean() {
        let cmd: Cmd<i32> = Cmd::none();
        let mapped: Cmd<String> = cmd.map(|n| n.to_string());
        assert!(!mapped.dirty);
    }

    #[test]
    fn batch_is_dirty() {
        let cmd: Cmd<i32> = Cmd::batch(vec![Action::None, Action::Quit]);
        assert!(cmd.dirty);
        assert!(matches!(cmd.action, Action::Batch(_)));
    }
}
