//! osteak — TEA (The Elm Architecture) for ratatui
//!
//! You keep your event loop. osteak provides the structure.
//!
//! # Architecture
//!
//! - **Model**: Your application state (the `App` struct)
//! - **Message**: An enum of everything that can happen
//! - **Update**: `fn update(&mut self, msg: Msg) -> Action<Msg>`
//! - **View**: `fn view(&self, frame: &mut Frame)`
//!
//! osteak does NOT own your event loop. You call `update()` and `view()`
//! when you want. This lets you integrate with any async runtime, any
//! event source, any rendering strategy.

mod runner;

use ratatui::Frame;

/// The core TEA trait. Implement this on your application state.
///
/// Your `Model` (the struct implementing this trait) is the single source
/// of truth. All state lives here. Update takes `&mut self` — no cloning.
pub trait Tea {
    /// The message type — everything that can happen in your app.
    type Msg;

    /// Process a message, mutate state, return an action.
    ///
    /// Returns `(Action<Msg>, bool)` where the bool indicates whether
    /// the view is dirty and needs re-rendering.
    fn update(&mut self, msg: Self::Msg) -> (Action<Self::Msg>, bool);

    /// Render the current state. You get full `Frame` access —
    /// multiple areas, StatefulWidget, whatever you need.
    fn view(&self, frame: &mut Frame);
}

/// What to do after an update.
pub enum Action<Msg> {
    /// Nothing to do.
    None,

    /// Spawn an async task. The future resolves to a message
    /// that gets fed back into `update()`.
    ///
    /// The caller decides HOW to spawn (tokio::spawn, spawn_blocking, etc).
    /// osteak just tells you what to spawn.
    Task(Pin<Box<dyn std::future::Future<Output = Msg> + Send + 'static>>),

    /// Spawn multiple tasks.
    Batch(Vec<Action<Msg>>),

    /// Quit the application.
    Quit,
}

/// A subscription — a stream of messages from an external source.
///
/// Subscriptions are how you wire channels, file watchers, timers,
/// or any other `Stream<Item = Msg>` into the TEA loop.
pub trait Subscription<Msg> {
    /// Return streams that produce messages. Called once at startup
    /// and whenever you want to re-subscribe (e.g., after a state change).
    fn subscriptions(&self) -> Vec<Box<dyn futures_core::Stream<Item = Msg> + Send + Unpin>>;
}

use std::pin::Pin;
