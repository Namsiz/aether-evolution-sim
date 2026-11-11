/// Integration test: lamport causal ordering across asynchronous frame advancement.
///
/// Validates the core invariant of HierarchicalClock: an event broadcast from frame A
/// to frame B can never be processed by B before A's broadcast version is established.
#[path = "../src/lib.rs"]
mod aether_clock;

use aether_clock::{AnchorEvent, FrameId, HierarchicalClock};

/// Helper: create a deterministic fake clock that advances by 50ms per call.
fn make_step_clock(step_ms: u64) -> (Box<dyn Fn() -> std::time::Instant + Send + Sync>, std::sync::Arc<std::sync::Mutex<u64>>) {
    let counter = std::sync::Arc::new(std::sync::Mutex::new(0u64));
    let c = counter.clone();
    let src: Box<dyn Fn() -> std::time::Instant + Send + Sync> = Box::new(move || {
        let mut n = c.lock().unwrap();
        let base = std::time::Instant::now() + std::time::Duration::from_millis(*n * step_ms);
        *n += 1;
        base
    });
    (src, counter)
}

fn tick_fake_clock(counter: &std::sync::Arc<std::sync::Mutex<u64>>, steps: u64) {
    let mut n = counter.lock().unwrap();
    *n += steps;
}

#[test]
fn broadcast_then_advance_verifies_version_catchup() {
    let (src, counter) = make_step_clock(50);
    let mut clock = HierarchicalClock::with_time_source(src);
    clock.register(1);
    clock.register(2);
    clock.set_children(1, vec![2]);

    // Advance frame 1 a bit
    clock.advance(1, 1.0);
    assert_eq!(clock.clock(1).0, 10);

    // Broadcast — version should be 11 (after the advance bumped to 10, then +1)
    let evt = clock.broadcast(1, b"hello".to_vec());
    assert_eq!(evt.version, 11);

    // Drive fake clock past the 50ms enqueue delay
    tick_fake_clock(&counter, 2);

    // Frame 2 should now receive the event
    let fired = clock.process_pending(2);
    assert_eq!(fired.len(), 1);
    assert_eq!(fired[0].0.version, 11);
}

#[test]
fn async_advancement_preserves_no_future_violation() {
    let (src, counter) = make_step_clock(50);
    let mut clock = HierarchicalClock::with_time_source(src);
    clock.register(10);
    clock.register(20);
    clock.set_children(10, vec![20]);

    // Frame 20 runs far ahead
    clock.advance(20, 100.0); // clock = 1000
    assert_eq!(clock.clock(20).0, 1000);

    // Frame 10 broadcasts — its version will be 1 (first broadcast)
    let _evt = clock.broadcast(10, b"sync".to_vec());
    assert_eq!(clock.clock(10).0, 1);

    // Drive fake clock to deliver the event
    tick_fake_clock(&counter, 3);

    // Frame 20 processes — should NOT update its clock downward (already at 1000)
    let fired = clock.process_pending(20);
    assert_eq!(fired.len(), 1);
    assert_eq!(fired[0].0.version, 1);
    assert_eq!(clock.clock(20).0, 1000); // stays at 1000, not reduced
}

#[test]
fn multiple_events_ordered_by_version() {
    let (src, counter) = make_step_clock(50);
    let mut clock = HierarchicalClock::with_time_source(src);
    clock.register(1);
    clock.register(2);
    clock.set_children(1, vec![2]);

    // Three sequential broadcasts
    let e1 = clock.broadcast(1, b"a".to_vec());
    let e2 = clock.broadcast(1, b"b".to_vec());
    let e3 = clock.broadcast(1, b"c".to_vec());

    assert_eq!(e1.version, 1);
    assert_eq!(e2.version, 2);
    assert_eq!(e3.version, 3);

    // Deliver all
    tick_fake_clock(&counter, 5);
    let fired = clock.process_pending(2);
    assert_eq!(fired.len(), 3);
    assert_eq!(fired[0].0.version, 1);
    assert_eq!(fired[1].0.version, 2);
    assert_eq!(fired[2].0.version, 3);
}
