#![warn(missing_docs)]
#![forbid(unsafe_code)]
//! # osteak — TEA (The Elm Architecture) for ratatui
//!
//! You keep your event loop. osteak provides the structure.
//!
//! ## The Problem
//!
//! Every ratatui app that grows past demo complexity hits the "bag of
//! booleans" wall. State scatters across fields, transitions become
//! implicit, and race conditions emerge when async work outlives UI
//! signals.
//!
//! ## The Solution
//!
//! osteak brings the [Elm Architecture] to ratatui:
//!
//! - **Model** — your application state (the struct implementing [`Tea`])
//! - **Message** — an enum of everything that can happen ([`Tea::Msg`])
//! - **Update** — process a message, mutate state, return a [`Cmd`]
//! - **View** — render the current state to a ratatui [`Frame`]
//!
//! ## You Keep Your Event Loop
//!
//! osteak does **not** own your event loop. You call [`Tea::update`] and
//! [`Tea::view`] when you want. This lets you integrate with any async
//! runtime, any event source, any rendering strategy.
//!
//! For simple apps, the optional [`runner`] module provides a ready-made
//! event loop powered by crossterm and tokio.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use osteak::{Tea, Cmd, Action};
//! use ratatui::Frame;
//! use ratatui::widgets::Paragraph;
//!
//! struct Counter { count: i32 }
//!
//! enum Msg { Increment, Decrement, Quit }
//!
//! impl Tea for Counter {
//!     type Msg = Msg;
//!
//!     fn update(&mut self, msg: Msg) -> Cmd<Msg> {
//!         match msg {
//!             Msg::Increment => { self.count += 1; Cmd::dirty() }
//!             Msg::Decrement => { self.count -= 1; Cmd::dirty() }
//!             Msg::Quit => Cmd::quit(),
//!         }
//!     }
//!
//!     fn view(&mut self, frame: &mut Frame) {
//!         let text = format!("Count: {}", self.count);
//!         frame.render_widget(Paragraph::new(text), frame.area());
//!     }
//! }
//! ```
//!
//! [Elm Architecture]: https://guide.elm-lang.org/architecture/
//! [`Frame`]: ratatui::Frame

mod action;
mod cmd;
pub mod runner;
mod sub;

pub use action::Action;
pub use cmd::Cmd;
pub use sub::Sub;

use ratatui::Frame;

/// The core TEA trait. Implement this on your application state.
///
/// Your model (the struct implementing this trait) is the single source
/// of truth. All state lives here. [`update`](Tea::update) takes `&mut self` —
/// no cloning per event.
///
/// # Associated Types
///
/// - [`Msg`](Tea::Msg) — the message type. Must be `Send + 'static`
///   because [`Action::Task`] futures need to send messages across threads.
///
/// # Required Methods
///
/// - [`update`](Tea::update) — process a message, mutate state, return a [`Cmd`].
/// - [`view`](Tea::view) — render to a ratatui [`Frame`]. Takes `&mut self`
///   because ratatui's `StatefulWidget` pattern requires mutable access to
///   render state (scroll positions, list selection, etc.).
///
/// # Optional Methods
///
/// - [`init`](Tea::init) — return an [`Action`] to run at startup (default: no-op).
/// - [`subscriptions`](Tea::subscriptions) — return active [`Sub`]scriptions
///   for external event sources (default: none).
///
/// # Terminal Ownership
///
/// Your model must **not** own the `Terminal`. Keep the terminal in your
/// event loop and pass the [`Frame`] to [`view`](Tea::view) via
/// `terminal.draw(|f| model.view(f))`. This separation is enforced by the
/// borrow checker and is the universal ratatui pattern.
pub trait Tea {
    /// The message type — everything that can happen in your app.
    ///
    /// Typically an enum with a variant per event kind. Must be `Send + 'static`
    /// so that [`Action::Task`] futures can produce messages from other threads.
    type Msg: Send + 'static;

    /// Process a message, mutate state, return a [`Cmd`].
    ///
    /// The returned [`Cmd`] tells the caller what side effect to perform
    /// (if any) and whether the view needs re-rendering.
    ///
    /// # Examples
    ///
    /// ```
    /// use osteak::{Tea, Cmd};
    /// use ratatui::Frame;
    ///
    /// struct App { count: i32 }
    /// enum Msg { Inc }
    ///
    /// impl Tea for App {
    ///     type Msg = Msg;
    ///     fn update(&mut self, msg: Msg) -> Cmd<Msg> {
    ///         match msg {
    ///             Msg::Inc => { self.count += 1; Cmd::dirty() }
    ///         }
    ///     }
    ///     fn view(&mut self, frame: &mut Frame) {}
    /// }
    /// ```
    fn update(&mut self, msg: Self::Msg) -> Cmd<Self::Msg>;

    /// Render the current state to a ratatui [`Frame`].
    ///
    /// You get full `Frame` access — multiple layout areas,
    /// `StatefulWidget`, whatever you need. This is not limited to
    /// returning a single `Widget`.
    ///
    /// Takes `&mut self` because ratatui's `StatefulWidget::render`
    /// requires `&mut State` for things like scroll positions and
    /// selection offsets. Keep this render-side state in your model
    /// alongside application state.
    fn view(&mut self, frame: &mut Frame);

    /// Return an [`Action`] to run at startup.
    ///
    /// Override this to kick off initial data loads, start timers,
    /// or perform other setup work. The default does nothing.
    fn init(&mut self) -> Action<Self::Msg> {
        Action::None
    }

    /// Return active subscriptions for external event sources.
    ///
    /// Each [`Sub`] pairs a stable identity string with a
    /// [`Stream`](futures_core::Stream) that produces messages. The
    /// optional runner uses the identity to diff subscriptions between
    /// updates — starting new ones and cancelling stale ones.
    ///
    /// For manual event loops, you manage subscription lifecycle yourself.
    ///
    /// The default returns no subscriptions.
    fn subscriptions(&self) -> Vec<Sub<Self::Msg>> {
        vec![]
    }
}
