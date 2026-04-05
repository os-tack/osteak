//! Optional runner — a convenience for simple TEA apps.
//!
//! The runner handles terminal setup/teardown, event polling, task
//! spawning, subscription management, and the draw loop. For complex
//! apps with existing event loops or multiple event sources, skip this
//! module and call [`Tea::update`](crate::Tea::update) /
//! [`Tea::view`](crate::Tea::view) directly.
//!
//! Requires features: `crossterm-backend` + `tokio-runtime` (both
//! enabled by default).
//!
//! # Examples
//!
//! ```rust,no_run
//! use osteak::{Tea, Cmd, Action};
//! use ratatui::Frame;
//! use ratatui::widgets::Paragraph;
//! use crossterm::event::{Event, KeyCode, KeyEventKind};
//!
//! struct App { count: i32 }
//! enum Msg { Inc, Quit }
//!
//! impl Tea for App {
//!     type Msg = Msg;
//!     fn update(&mut self, msg: Msg) -> Cmd<Msg> {
//!         match msg {
//!             Msg::Inc => { self.count += 1; Cmd::dirty() }
//!             Msg::Quit => Cmd::quit(),
//!         }
//!     }
//!     fn view(&mut self, frame: &mut Frame) {
//!         frame.render_widget(
//!             Paragraph::new(format!("{}", self.count)),
//!             frame.area(),
//!         );
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> std::io::Result<()> {
//!     osteak::runner::run(App { count: 0 }, |ev| {
//!         if let Event::Key(k) = ev {
//!             if k.kind != KeyEventKind::Press { return None; }
//!             match k.code {
//!                 KeyCode::Up => Some(Msg::Inc),
//!                 KeyCode::Char('q') => Some(Msg::Quit),
//!                 _ => None,
//!             }
//!         } else { None }
//!     }).await
//! }
//! ```

#[cfg(all(feature = "crossterm-backend", feature = "tokio-runtime"))]
mod inner {
    use std::collections::HashMap;
    use std::io;

    use crossterm::event::{Event, EventStream};
    use futures::StreamExt;
    use tokio::sync::mpsc;
    use tokio::task::JoinHandle;

    use crate::{Action, Tea};

    /// Run a TEA app with sensible defaults.
    ///
    /// This sets up the terminal, polls crossterm events, spawns tasks,
    /// manages subscriptions, and draws frames. When the model returns
    /// [`Action::Quit`], the terminal is restored and the function returns.
    ///
    /// The `event_to_msg` function maps crossterm [`Event`]s to your
    /// message type. Return `None` to ignore an event.
    ///
    /// # Errors
    ///
    /// Returns an error if terminal initialization, drawing, or
    /// restoration fails.
    pub async fn run<M, F>(model: M, event_to_msg: F) -> io::Result<()>
    where
        M: Tea,
        F: Fn(Event) -> Option<M::Msg> + Send + 'static,
    {
        Runner::new(model, event_to_msg).run().await
    }

    /// A configurable TEA runner.
    ///
    /// Use this instead of [`run`] when you need to customize behavior
    /// like the event-to-message mapping.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use osteak::{Tea, Cmd};
    /// # use ratatui::Frame;
    /// # struct App;
    /// # enum Msg {}
    /// # impl Tea for App {
    /// #     type Msg = Msg;
    /// #     fn update(&mut self, _: Msg) -> Cmd<Msg> { unreachable!() }
    /// #     fn view(&mut self, _: &mut Frame) {}
    /// # }
    /// use osteak::runner::Runner;
    ///
    /// # async fn example() -> std::io::Result<()> {
    /// Runner::new(App, |_| None)
    ///     .run()
    ///     .await
    /// # }
    /// ```
    pub struct Runner<M: Tea, F> {
        model: M,
        event_to_msg: F,
    }

    impl<M, F> Runner<M, F>
    where
        M: Tea,
        F: Fn(Event) -> Option<M::Msg> + Send + 'static,
    {
        /// Create a new runner.
        pub fn new(model: M, event_to_msg: F) -> Self {
            Runner {
                model,
                event_to_msg,
            }
        }

        /// Run the TEA event loop.
        ///
        /// This takes ownership of the runner. The terminal is initialized
        /// before the loop starts and restored (even on panic) when it ends.
        pub async fn run(mut self) -> io::Result<()> {
            let mut terminal = ratatui::try_init()?;

            // Ensure terminal is restored on panic or early return.
            let result = self.event_loop(&mut terminal).await;

            ratatui::try_restore()?;
            result
        }

        async fn event_loop(
            &mut self,
            terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
        ) -> io::Result<()> {
            // Channel for task results feeding back into the update loop.
            let (task_tx, mut task_rx) = mpsc::unbounded_channel::<M::Msg>();

            // Process init action.
            let init_action = self.model.init();
            Self::process_action(init_action, &task_tx);

            // Initial render.
            terminal.draw(|f| self.model.view(f))?;

            // Set up event stream.
            let mut events = EventStream::new();

            // Active subscriptions tracked by id.
            let mut active_subs: HashMap<&'static str, JoinHandle<()>> = HashMap::new();
            let sub_tx = task_tx.clone();
            self.sync_subscriptions(&mut active_subs, &sub_tx);

            loop {
                // Select across all event sources.
                let msg = tokio::select! {
                    // Crossterm terminal events.
                    Some(Ok(ev)) = events.next() => {
                        (self.event_to_msg)(ev)
                    }
                    // Task completion results.
                    Some(msg) = task_rx.recv() => {
                        Some(msg)
                    }
                };

                let Some(msg) = msg else { continue };

                // Update.
                let cmd = self.model.update(msg);

                // Process the action.
                let quit = matches!(cmd.action, Action::Quit);
                Self::process_action(cmd.action, &task_tx);

                if quit {
                    break;
                }

                // Re-sync subscriptions after state change.
                self.sync_subscriptions(&mut active_subs, &sub_tx);

                // Re-render if dirty.
                if cmd.dirty {
                    terminal.draw(|f| self.model.view(f))?;
                }
            }

            // Cancel all active subscriptions.
            for (_, handle) in active_subs.drain() {
                handle.abort();
            }

            Ok(())
        }

        fn process_action(action: Action<M::Msg>, task_tx: &mpsc::UnboundedSender<M::Msg>) {
            match action {
                Action::None | Action::Quit => {}
                Action::Task(fut) => {
                    let tx = task_tx.clone();
                    tokio::spawn(async move {
                        let msg = fut.await;
                        let _ = tx.send(msg);
                    });
                }
                Action::Batch(actions) => {
                    for action in actions {
                        Self::process_action(action, task_tx);
                    }
                }
            }
        }

        fn sync_subscriptions(
            &self,
            active: &mut HashMap<&'static str, JoinHandle<()>>,
            task_tx: &mpsc::UnboundedSender<M::Msg>,
        ) {
            let desired = self.model.subscriptions();
            let desired_ids: std::collections::HashSet<&'static str> =
                desired.iter().map(|s| s.id).collect();

            // Cancel subscriptions that are no longer wanted.
            active.retain(|id, handle| {
                if desired_ids.contains(id) {
                    true
                } else {
                    handle.abort();
                    false
                }
            });

            // Start new subscriptions.
            for sub in desired {
                if active.contains_key(sub.id) {
                    continue;
                }
                let tx = task_tx.clone();
                let id = sub.id;
                let handle = tokio::spawn(async move {
                    let mut stream = sub.stream;
                    while let Some(msg) = StreamExt::next(&mut stream).await {
                        if tx.send(msg).is_err() {
                            break;
                        }
                    }
                });
                active.insert(id, handle);
            }
        }
    }
}

#[cfg(all(feature = "crossterm-backend", feature = "tokio-runtime"))]
pub use inner::*;
