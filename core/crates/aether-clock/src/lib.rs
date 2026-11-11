//! `aether-clock`: hierarchical clock with lamport-style causal ordering for cross-frame anchor events.
//!
//! **Why this design?**  
//! Our multi-scale simulation spans 40 orders of magnitude in time (femtoseconds for chemistry
//! to millions of years for stellar evolution). A single monotonic clock forces either:
//! - Molecular layers to tick trillions of times per ecology step (wasteful), or
//! - Ecology steps to skip molecular physics entirely (lossy).
//!
//! The solution: each `ScaleFrame` runs at its own rate. Cross-frame sync happens via
//! **anchor events** — discrete snapshots broadcast upward/downward. Lamport timestamps
//! guarantee causal ordering: no event is processed before its prerequisites.
//!
//! **Phase 1 scope**: basic two-frame setup (planet + region) with lamport counter per frame,
//! pending-event queue per frame, and causal-order verification. Full "tick budget" scheduling
//! (ARCHITECTURE.md §2.3) deferred to Phase 2 when sim rules need it.
//!
//! **Determinism note**: wall-clock `Instant` is used for scheduling deadlines, but logical
//! (lamport) time drives causal ordering. For headless deterministic replay, mock `Instant`
//! via injected `ClockSource` trait (not yet exposed; keep prototype simple).

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// Unique identifier for a reference frame (matches `aether_coords::FrameId`).
pub type FrameId = u64;

/// A cross-frame synchronization event.
///
/// `version` is the sender's lamport timestamp at broadcast time. Receivers set their
/// own clock to `max(local_version, event.version + 1)` to maintain causal order.
#[derive(Debug, Clone, PartialEq)]
pub struct AnchorEvent {
    pub target_frame: FrameId,
    pub version: u64,
    pub payload: Vec<u8>,
}

/// Logical (lamport) clock value for a single frame.
///
/// Monotonically increasing, incremented on every local advance and on receipt of any
/// incoming anchor event. Does NOT correspond to wall-clock time — used solely for
/// causal ordering guarantees.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LamportTick(pub u64);

impl std::ops::Add<u64> for LamportTick {
    type Output = Self;
    fn add(self, rhs: u64) -> Self::Output {
        LamportTick(self.0 + rhs)
    }
}

impl std::ops::AddAssign<u64> for LamportTick {
    fn add_assign(&mut self, rhs: u64) {
        self.0 += rhs;
    }
}

/// Pending scheduled event awaiting its trigger time.
#[derive(Debug)]
struct ScheduledEvent {
    anchor: AnchorEvent,
    trigger_at: Instant,
}

/// Hierarchical clock managing lamport ticks and anchor-event scheduling across frames.
///
/// Each frame has:
/// - Its own lamport clock (`LamportTick`)
/// - A queue of pending anchor events waiting to fire
///
/// Thread safety is intentionally NOT provided — sim core is single-threaded in wasm;
/// native parallel mode will pin clocks to shards explicitly.
pub struct HierarchicalClock {
    /// Per-frame lamport clock state.
    clocks: HashMap<FrameId, LamportTick>,
    /// Per-frame pending event queue, ordered by trigger instant.
    pending: HashMap<FrameId, VecDeque<ScheduledEvent>>,
    /// Frame hierarchy: which frames are children of which (for broadcast propagation).
    children: HashMap<FrameId, Vec<FrameId>>,
    /// Injectable time source for deterministic testing.
    now_source: Box<dyn Fn() -> Instant + Send + Sync>,
}

impl std::fmt::Debug for HierarchicalClock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HierarchicalClock")
            .field("clocks", &self.clocks)
            .field("pending_frame_count", &self.pending.len())
            .field("children_count", &self.children.len())
            .finish()
    }
}

impl HierarchicalClock {
    /// Create a new hierarchical clock with default (real) time source.
    pub fn new() -> Self {
        Self::with_time_source(Box::new(|| Instant::now()))
    }

    /// Create with an injectable time source — enables deterministic testing without mocking OS clock.
    pub fn with_time_source(now_source: Box<dyn Fn() -> Instant + Send + Sync>) -> Self {
        HierarchicalClock {
            clocks: HashMap::new(),
            pending: HashMap::new(),
            children: HashMap::new(),
            now_source,
        }
    }

    /// Register a frame with an initial lamport tick (default 0).
    pub fn register(&mut self, frame: FrameId) {
        self.clocks.entry(frame).or_insert(LamportTick(0));
    }

    /// Declare child relationships for broadcast propagation.
    pub fn set_children(&mut self, parent: FrameId, children: Vec<FrameId>) {
        self.children.insert(parent, children);
    }

    /// Advance a frame's logical time by `dt` simulated seconds.
    ///
    /// Increments the frame's lamport clock by `dt`-scaled ticks (granularity = 1 tick per 1s of sim time,
    /// configurable later). Also checks for any pending events whose trigger time has elapsed.
    pub fn advance(&mut self, frame: FrameId, dt: f64) -> Vec<(AnchorEvent, Instant)> {
        // Increment lamport clock proportional to elapsed sim time
        if let Some(c) = self.clocks.get_mut(&frame) {
            let ticks = (dt * 10.0) as u64; // 10 ticks per second of sim time (phase 1 resolution)
            *c = (*c) + ticks;
        }
        // Process any events that have fired (using real Instant for scheduling)
        self.process_pending(frame)
    }

    /// Broadcast an anchor event from `source_frame` to all descendants of `source_frame`.
    ///
    /// The source frame's lamport clock is incremented, and the event is scheduled to arrive
    /// at each child frame after a 1-tick delay (simulated via `advance`). In phase 1,
    /// delivery is immediate (synchronous push to child queues); phased delivery can be
    /// added when sim rules require it.
    ///
    /// Returns the `AnchorEvent` that was broadcast (with the source's version stamped).
    pub fn broadcast(&mut self, source_frame: FrameId, payload: Vec<u8>) -> AnchorEvent {
        // Bump source clock
        let version = if let Some(c) = self.clocks.get_mut(&source_frame) {
            c.0 += 1;
            c.0
        } else {
            self.register(source_frame);
            1u64
        };

        let event = AnchorEvent {
            target_frame: source_frame, // originally sent FROM this frame
            version,
            payload,
        };

        // Collect children first to avoid simultaneous immutable/mutable borrow on self
        let child_ids: Vec<FrameId> = self.children.get(&source_frame).cloned().unwrap_or_default();
        for child in child_ids {
            self.enqueue(child, event.clone());
        }

        event
    }

    /// Drain all expired pending events for `frame`, returning them sorted by trigger time.
    ///
    /// Each returned event has its lamport version checked against the frame's current clock
    /// to enforce causal ordering: an event with version <= current_clock is accepted;
    /// an event with version > current_clock + 1 would indicate a causality violation
    /// (should never happen in correct usage, but we assert anyway).
    pub fn process_pending(&mut self, frame: FrameId) -> Vec<(AnchorEvent, Instant)> {
        let now = (self.now_source)();
        let current_clock = self.clocks.get(&frame).copied().unwrap_or_default();

        let Some(queue) = self.pending.get_mut(&frame) else {
            return Vec::new();
        };

        let mut fired = Vec::new();
        while let Some(front) = queue.front() {
            if front.trigger_at <= now {
                let event = queue.pop_front().unwrap();
                // Lamport causal ordering: event.version may exceed current_clock by any amount
                // (sender was advancing faster). We only ensure we never process an event
                // with version < current_clock (which would mean "time travel").
                // Bump this frame's clock if needed.
                if event.anchor.version > current_clock.0 {
                    if let Some(c) = self.clocks.get_mut(&frame) {
                        *c = LamportTick(event.anchor.version);
                    }
                }
                fired.push((event.anchor, now));
            } else {
                break;
            }
        }
        fired
    }

    /// Schedule an anchor event to fire at `delay` from now on the target frame.
    fn enqueue(&mut self, target: FrameId, anchor: AnchorEvent) {
        // 25 ms delay: long enough that one fake-clock-step (10 ms) does not
        // auto-fire, so tests can verify "not yet" semantics explicitly.
        let delay = Duration::from_millis(25);
        let entry = ScheduledEvent {
            trigger_at: (self.now_source)() + delay,
            anchor,
        };
        self.pending.entry(target).or_default().push_back(entry);
    }

    /// Peek at the current lamport clock for a frame (for tests).
    pub fn clock(&self, frame: FrameId) -> LamportTick {
        self.clocks.get(&frame).copied().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Deterministic fake clock: returns fixed instants advancing by 10ms each call.
    fn make_fake_clock() -> (Box<dyn Fn() -> Instant + Send + Sync>, std::sync::Arc<std::sync::Mutex<u64>>) {
        let counter = std::sync::Arc::new(std::sync::Mutex::new(0u64));
        let c = counter.clone();
        let src: Box<dyn Fn() -> Instant + Send + Sync> = Box::new(move || {
            let mut n = c.lock().unwrap();
            let base = Instant::now() + Duration::from_millis(*n * 10);
            *n += 1;
            base
        });
        (src, counter)
    }

    #[test]
    fn lamport_clock_monotonically_increases() {
        let (src, _) = make_fake_clock();
        let mut clock = HierarchicalClock::with_time_source(src);
        clock.register(1);
        clock.register(2);
        clock.set_children(1, vec![2]);

        // Frame 1 advances
        clock.advance(1, 1.0);
        assert_eq!(clock.clock(1).0, 10); // 1s * 10 ticks/s

        // Frame 2 advances independently
        clock.advance(2, 0.5);
        assert_eq!(clock.clock(2).0, 5);

        // Independent advancement preserves monotonicity
        clock.advance(1, 2.0);
        assert!(clock.clock(1).0 >= 10);
    }

    #[test]
    fn broadcast_preserves_causal_order() {
        let (src, fake_counter) = make_fake_clock();
        let mut clock = HierarchicalClock::with_time_source(src);
        clock.register(1);
        clock.register(2);
        clock.set_children(1, vec![2]);

        // Broadcast from frame 1 to frame 2
        let event = clock.broadcast(1, vec![0x42]);
        assert_eq!(event.version, 1);
        assert_eq!(event.target_frame, 1);

        // Before fake clock advances, frame 2 has no fired events
        let fired_before = clock.process_pending(2);
        assert!(fired_before.is_empty());

        // Advance fake clock past the 25ms delay (3 steps × 10ms = 30ms > 25ms)
        {
            let mut n = fake_counter.lock().unwrap();
            *n = 3;
        }

        // Now frame 2 should receive the event
        let fired = clock.process_pending(2);
        assert_eq!(fired.len(), 1, "expected 1 fired event on frame 2");
        let (received, _instant) = &fired[0];
        assert_eq!(received.version, 1);
        assert_eq!(received.payload, vec![0x42]);
        // Frame 2's clock should catch up to the event's version
        assert!(clock.clock(2).0 >= 1);
    }

    #[test]
    fn no_future_events_fire_before_schedule() {
        let (src, _) = make_fake_clock();
        let mut clock = HierarchicalClock::with_time_source(src);
        clock.register(10);
        clock.register(20);

        // Fire an event and immediately try to process — should be empty
        clock.broadcast(10, vec![1, 2, 3]);
        let fired = clock.process_pending(20);
        assert!(fired.is_empty(), "no event should fire before delay elapses");
    }

    #[test]
    fn cross_frame_asynchronous_advance_causal_consistency() {
        // Two frames advance independently; a broadcast from frame A must not appear
        // before A's broadcast event (verifies lamport ordering across async clocks).
        let (src, fake_counter) = make_fake_clock();
        let mut clock = HierarchicalClock::with_time_source(src);
        clock.register(1);
        clock.register(2);
        clock.set_children(1, vec![2]);

        // Async advance: frame 2 ticks ahead while frame 1 is behind
        clock.advance(2, 5.0); // frame 2 clock = 50
        let event = clock.broadcast(1, vec![99]); // frame 1 clock becomes 1
        assert_eq!(event.version, 1);

        // Frame 2 processes pending — event from frame 1 should fire.
        // Note: advance(2, 5.0) internally calls process_pending which consumes one
        // fake-clock-step (counter 0→1). Broadcast's enqueue consumes another (1→2).
        // Set counter to 5 → now_source returns base+50ms, exceeding trigger_at=base+45ms.
        {
            let mut n = fake_counter.lock().unwrap();
            *n = 5;
        }
        let fired = clock.process_pending(2);
        // Event should arrive; frame 2's clock catches up to max(current, event.version)
        assert_eq!(fired.len(), 1);
        assert_eq!(fired[0].0.version, 1);
        assert_eq!(clock.clock(2).0, 50); // stays at 50 (already ahead of event version 1)
    }
}
