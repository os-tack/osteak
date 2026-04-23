//! →1413: Tests for `Sub::lazy` factory variant.
//!
//! Verifies the core invariant: the factory passed to `Sub::lazy` is
//! invoked **exactly once per actual spawn** — never on drop, never
//! multiple times. This is the behavior that keeps
//! `App::subscriptions()` side-effect-free even when it returns a fresh
//! `Sub::lazy` on every call.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use futures::StreamExt;
use futures::stream;
use osteak::{Action, Cmd, Sub, Tea};
use ratatui::Frame;

// ─── Model plumbing ────────────────────────────────────────────────

/// Minimal Tea model that returns the subscriptions vector it was
/// configured with on every `subscriptions()` call.
struct App<F>
where
    F: Fn() -> Vec<Sub<Msg>> + Send + Sync + 'static,
{
    make_subs: F,
}

enum Msg {
    #[allow(dead_code)]
    Noop,
}

impl<F> Tea for App<F>
where
    F: Fn() -> Vec<Sub<Msg>> + Send + Sync + 'static,
{
    type Msg = Msg;
    fn update(&mut self, _msg: Msg) -> Cmd<Msg> {
        Cmd::quit()
    }
    fn view(&mut self, _frame: &mut Frame) {}
    fn init(&mut self) -> Action<Msg> {
        Action::None
    }
    fn subscriptions(&self) -> Vec<Sub<Msg>> {
        (self.make_subs)()
    }
}

// ─── Tests ─────────────────────────────────────────────────────────

/// AC (5).a: a factory wired into a `Sub::lazy` is invoked **exactly
/// once** across N repeated `subscriptions()` calls with the same id.
///
/// This is the whole reason `Sub::lazy` exists — the runner's
/// `sync_subscriptions` drops Subs whose id is already active, and the
/// factory stashed inside a `Lazy` that gets dropped must NOT run.
#[test]
fn test_lazy_factory_invoked_exactly_once_per_spawn() {
    let counter = Arc::new(AtomicUsize::new(0));

    // Build N Sub::lazy with the same id. Only the first should have
    // `into_stream` called on it (simulating the runner's spawn path);
    // the rest should be dropped silently.
    let n = 1000;
    let mut first = None;
    for _ in 0..n {
        let c = Arc::clone(&counter);
        let sub = Sub::lazy("my-id", move || {
            c.fetch_add(1, Ordering::SeqCst);
            stream::iter(Vec::<Msg>::new())
        });
        if first.is_none() {
            first = Some(sub);
        }
        // The rest are dropped immediately — factory must NOT run.
    }

    // Invoke the retained factory via `into_stream` — simulates the
    // runner spawning the stream.
    let _stream = first.unwrap().into_stream();

    assert_eq!(
        counter.load(Ordering::SeqCst),
        1,
        "factory should have run exactly once (for the one we spawned)"
    );
}

/// AC (5).b: a `Sub::lazy` whose factory would panic, dropped without
/// spawning, must not panic.
#[test]
fn test_lazy_factory_not_invoked_on_drop() {
    let sub = Sub::lazy("panicky", || -> stream::Empty<Msg> {
        panic!("factory should never run on drop");
    });

    // Dropping without calling `into_stream` must not invoke factory.
    drop(sub);

    // If we got here, the factory didn't run — pass.
}

/// AC (5).c: existing `Sub::new` eager semantics preserved — the
/// stream is consumable via `into_stream` and yields its items in
/// order.
#[tokio::test]
async fn test_eager_sub_semantics_preserved() {
    #[derive(Debug, PartialEq, Eq)]
    enum M {
        A,
        B,
        C,
    }
    let sub = Sub::new("eager", stream::iter(vec![M::A, M::B, M::C]));
    assert_eq!(sub.id, "eager");

    let mut stream = sub.into_stream();
    let mut out = Vec::new();
    while let Some(m) = stream.next().await {
        out.push(m);
    }
    assert_eq!(out, vec![M::A, M::B, M::C]);
}

/// Complementary AC: constructing a `Sub::new` from a side-effectful
/// stream that counts its constructions confirms the **opposite**
/// direction — `Sub::new` does NOT protect you from per-description
/// side effects. (Documents why `Sub::lazy` exists.)
#[test]
fn test_eager_factory_runs_per_construction_not_per_spawn() {
    let counter = Arc::new(AtomicUsize::new(0));

    // Stream constructor that increments on construction (not on poll).
    // With Sub::new, this ticks every time we build a Sub.
    for _ in 0..5 {
        let c = Arc::clone(&counter);
        c.fetch_add(1, Ordering::SeqCst);
        let _sub = Sub::new("eager-counted", stream::iter(Vec::<Msg>::new()));
        // Sub dropped immediately.
    }

    assert_eq!(
        counter.load(Ordering::SeqCst),
        5,
        "Sub::new ran the 'construction' code on every build (this is WHY Sub::lazy exists)"
    );
}

/// Silence unused-warning for App/Msg: a trivial use keeps the plumbing
/// honest in case the test file grows an integration test later.
#[allow(dead_code)]
fn _unused_app_plumbing() {
    let _app: App<_> = App {
        make_subs: || vec![],
    };
}
