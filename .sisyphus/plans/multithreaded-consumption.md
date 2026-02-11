# Multithreaded Consumption Feature

## TL;DR

> Add multithreaded consumption to Cazzmachine: launch 1-8 parallel consuming threads per doomscrolling phase, each with staggered durations from 1/2 to full budget. Each thread triggers consumption + notification. Buffer maintains 20 items per thread. Status displays active thread count. Vertical slider controls thread count.

> **Deliverables**:
> - Rust backend: parallel thread scheduler with configurable count
> - Frontend: vertical slider (1-8), thread count status display
> - Notifications: per-thread completion notifications
> - Buffer: dynamic sizing based on thread count (20 × threads)
> - Crawler integration: trigger on slider change
>
> **Estimated Effort**: Medium
> **Parallel Execution**: YES - frontend and backend independent
> **Critical Path**: Rust thread scheduler → Frontend UI → Integration

---

## Context

### Original Request
User wants to implement "multithreaded consumption" where:
- Multiple "consuming threads" launched during doomscrolling phase
- Each thread has different duration, running in parallel
- Each timer triggers: consuming action + notification
- Status shows how many threads active
- Vertical slider (1-8) controls thread count
- Buffer ensures 20 items per thread (dynamic sizing)
- Crawler runs when slider changes

### Interview Summary

**Key Discussions**:
- Thread durations: Range from 1/2 consumption budget to full budget, equally spaced
- Thread timing: All threads start at same time (parallel), NOT sequential
- Total consumption WILL exceed budget (by design)
- Notifications: Fire when each thread ends, not just at phase end
- Buffer sizing: 20 items × thread count (e.g., 4 threads = 80 items)
- Slider trigger: Moving slider immediately triggers crawler to maintain buffer

---

## Work Objectives

### Core Objective
Implement parallel multithreaded consumption where N threads (1-8) consume content simultaneously, each with staggered completion times. Each thread triggers both consumption action and notification.

### Concrete Deliverables
- Rust backend: `ConsumptionThread` struct, thread scheduler
- Tauri command: `set_consumption_threads(count)` + `ConsumeResult` per thread
- Frontend: vertical slider (1-8), thread count in status bar
- Notification: per-thread completion events
- Buffer logic: `buffer_items = 20 * thread_count`
- Crawler trigger: call crawler when slider changes

### Definition of Done
- [ ] Vertical slider (1-8) in UI controls thread count
- [ ] Status bar shows "Scrolling with N threads"
- [ ] N timers fire in parallel on doomscrolling phase start
- [ ] Each timer triggers: `consumePendingItems(budget_minutes)` + notification
- [ ] Thread durations range from budget/2 to budget, equally spaced
- [ ] Buffer maintains 20 × N items
- [ ] Crawler triggers when slider value changes
- [ ] All tests pass

### Must Have
- Vertical slider (1-8 range)
- Parallel thread execution (same start time)
- Staggered completion times (durations from 1/2 to full budget)
- Per-thread notifications
- Dynamic buffer sizing (20 × thread_count)
- Status display showing active thread count

### Must NOT Have
- Sequential thread execution (threads MUST run parallel)
- Single-threaded mode when slider > 1 (all active threads run)
- Notifications only at phase end (per-thread notifications required)

---

## Technical Design

### Thread Duration Algorithm

For N threads with total budget B:
```
Thread i duration = B/2 + (i-1) × (B/2)/(N-1)  for i = 1 to N
```

**Examples**:
- N=1: Duration = B (half to full with 1 thread = full)
- N=2: Thread 1 = B/2, Thread 2 = B
- N=4, B=5min: 2.5min, 3.33min, 4.17min, 5min
- N=8, B=5min: 2.5min, 2.86min, 3.21min, 3.57min, 3.93min, 4.29min, 4.64min, 5min

**Key insight**: Total consumption = N × B (approximately) - exceeds budget intentionally

### Interval Between Doomscrolling Phases

**Formulas:**
```
S(level) = 1 + 4 × (level-1)/9          // Scroll: 1→5 min linearly
A(level) = 0.02 + 0.89 × (level-1)/9   // Active%: 2%→91% linearly
W(level) = S × ((1/A) - 1)             // Standby derived from active%
```

| Level | Scroll | Standby | Total | Active% |
|-------|--------|---------|-------|---------|
| 1 | 1.0 min | 49.0 min | 50 min | **2%** |
| 2 | 1.4 min | 10.7 min | 12 min | 12% |
| 3 | 1.9 min | 6.8 min | 8.7 min | 22% |
| 4 | 2.3 min | 5.0 min | 7.4 min | 32% |
| 5 | 2.8 min | 3.9 min | 6.7 min | 42% |
| 6 | 3.2 min | 3.0 min | 6.3 min | 51% |
| 7 | 3.7 min | 2.3 min | 6.0 min | 61% |
| 8 | 4.1 min | 1.6 min | 5.8 min | 71% |
| 9 | 4.6 min | 1.0 min | 5.6 min | 81% |
| 10 | 5.0 min | 0.5 min | 5.5 min | **91%** |

**Key insight:** `W = S × ((1/A) - 1)` is convex (standby drops faster at high levels), creating a "front-loaded" consumption pattern at high levels.

### Crawler Trigger on Slider Change

The existing crawler (`crawler/scheduler.rs:65-75`) already has "skip if buffer sufficient" logic:
```rust
match self.db.get_pending_count() {
    Ok(n) if n < target_buffer => {
        self.crawl_cycle().await;  // Crawls only if below target
    }
    Ok(pending) => {
        log::info!("Buffer has {} unconsumed items - skipping crawl", pending);
    }
    ...
}
```

**Integration approach:**
- When slider changes: call `ensure_buffer(target = 20 * threads)`
- `ensure_buffer()` checks: `get_pending_count() < 20 * threads` → triggers crawl if needed
- Crawler automatically skips if buffer already sufficient (no-op)

### Buffer Sizing

```rust
let target_buffer = 20 * thread_count;
let current = db.get_pending_count();
if current < target_buffer {
    trigger_crawler();
}
```

### Parallel Execution Model

```
Phase Start
    │
    ├─ Thread 1 (duration: B/2) ──→ consume(budget/2) ──→ notify
    │
    ├─ Thread 2 (duration: B/2 + δ) ──→ consume(budget/2 + δ) ──→ notify
    │
    ├─ Thread 3 (duration: B/2 + 2δ) ──→ consume(budget/2 + 2δ) ──→ notify
    │
    └─ Thread N (duration: B) ──→ consume(budget) ──→ notify
```

---

## Verification Strategy

### Test Decision
- **Infrastructure exists**: YES (Jest in package.json)
- **Automated tests**: Tests-after (Rust unit tests, frontend integration)
- **Framework**: Jest

### Agent-Executed QA Scenarios

**Scenario: Multithreaded consumption triggers correct number of threads**
  Tool: Playwright
  Preconditions: App running, thread slider visible
  Steps:
    1. Set thread slider to 4
    2. Start doomscrolling phase
    3. Wait 10 seconds
    4. Verify status shows "Scrolling with 4 threads"
    5. Check database: 4 consumePendingItems calls recorded
  Expected Result: Status shows 4 threads, DB shows 4 consumption records
  Evidence: Screenshot of status, DB query results

**Scenario: Thread durations are staggered**
  Tool: Playwright + Bash
  Preconditions: Thread slider set to 3, phase running
  Steps:
    1. Start phase with stopwatch
    2. Capture completion timestamps for each notification
    3. Calculate intervals between completions
  Expected Result: 3 notifications at staggered intervals
  Evidence: Notification timestamps, completion times

**Scenario: Buffer scales with thread count**
  Tool: Bash
  Preconditions: DB with items
  Steps:
    1. Set threads to 2 → verify buffer target = 40
    2. Set threads to 4 → verify buffer target = 80
    3. Set threads to 8 → verify buffer target = 160
  Expected Result: Buffer target = 20 × thread_count

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Start Immediately):
├── Task 1: Rust - ConsumptionThread struct + thread scheduler
├── Task 2: Rust - set_consumption_threads command + buffer logic
└── Task 3: Frontend - Thread slider UI component

Wave 2 (After Task 2):
├── Task 4: Rust - Notification integration per thread
└── Task 5: Frontend - Status bar thread count display

Wave 3 (After Wave 1):
└── Task 6: Integration - Slider → crawler trigger
```

### Dependency Matrix

| Task | Depends On | Blocks | Can Parallelize With |
|------|------------|--------|---------------------|
| 1 | None | 4 | 2, 3 |
| 2 | None | 6 | 1, 3 |
| 3 | None | None | 1, 2 |
| 4 | 1 | None | 5 |
| 5 | 3 | None | 4 |
| 6 | 2, 3 | None | None (final) |

### Agent Dispatch Summary

| Wave | Tasks | Recommended Agents |
|------|-------|-------------------|
| 1 | 1, 2, 3 | visual-engineering (UI), ultrabrain (Rust scheduler) |
| 2 | 4, 5 | quick (notifications), frontend-ui-ux (status display) |
| 3 | 6 | task(category="visual-engineering", ...) |

---

## TODOs

- [ ] 1. Rust - ConsumptionThread struct and thread scheduler

  **What to do**:
  - Create `ConsumptionThread` struct with: id, duration, budget, status
  - Implement `ThreadScheduler` with:
    - `start_threads(budget_minutes: f64, thread_count: u8) -> Vec<ConsumptionThread>`
    - `get_thread_status() -> Vec<ConsumptionThread>`
    - Parallel execution using tokio::spawn
  - Each thread: spawn async task that waits on timer, then calls `consumePendingItems`
  - Return handles for tracking completion

  **Must NOT do**:
  - Use blocking sleep (must use tokio::time::sleep)
  - Block main executor

  **Recommended Agent Profile**:
  - **Category**: `ultrabrain`
    - Reason: Concurrent/parallel programming with tokio requires careful async handling
  - **Skills**: [`rust`, `tokio`]
    - `rust`: Core language for implementation
    - `tokio`: Async runtime for parallel thread scheduling

  **References**:
  - `src-tauri/src/notifications/mod.rs:29-52` - tokio::select pattern for async timers
  - `src-tauri/src/db/mod.rs:163-221` - consumePendingItems implementation

  **Acceptance Criteria**:
  - [ ] ConsumptionThread struct defined with id, duration, budget, status fields
  - [ ] ThreadScheduler::start_threads returns Vec<Handle>
  - [ ] All N threads start within 100ms of each other (parallel start)
  - [ ] Each thread fires at its designated duration (staggered completions)
  - [ ] Thread status tracked: pending, running, completed, failed

  **Agent-Executed QA Scenarios**:

  \`\`\`
  Scenario: Thread scheduler launches N threads in parallel
    Tool: Bash
    Preconditions: Rust test environment
    Steps:
      1. cargo test --lib thread_scheduler_parallel_start
      2. Measure time from start_threads call to all threads spawned
    Expected Result: All threads spawn within 100ms regardless of N (1-8)
    Evidence: Test output with timestamps

  Scenario: Thread durations are equally spaced from budget/2 to budget
    Tool: Bash
    Preconditions: Rust test with mock time
    Steps:
      1. Start 4 threads with budget=5min
      2. Record completion times
      3. Verify: t1≈2.5min, t2≈3.33min, t3≈4.17min, t4=5min
    Expected Result: Durations match formula B/2 + (i-1)*(B/2)/(N-1)
    Evidence: Test output with completion timestamps
  \`\`\`

  **Commit**: YES
  - Message: `feat(consumption): add multithreaded consumption scheduler`
  - Files: `src-tauri/src/consumption_threads.rs`, `src-tauri/src/lib.rs`

- [ ] 0.5. Rust - Update interval formulas for linear active%

  **What to do**:
  - Update `get_notify_interval()` in `notifications/mod.rs`:
    ```rust
    fn get_notify_interval(&self) -> Duration {
        let level = THROTTLE_LEVEL.load(std::sync::atomic::Ordering::Relaxed);
        // Scroll: 1→5 min linearly
        let scroll_minutes = 1.0 + 4.0 * ((level as f64 - 1.0) / 9.0);
        // Active%: 2%→91% linearly  
        let active_pct = 0.02 + 0.89 * ((level as f64 - 1.0) / 9.0);
        // Standby: W = S × ((1/A) - 1)
        let standby_minutes = scroll_minutes * ((1.0 / active_pct) - 1.0);
        Duration::from_secs((standby_minutes * 60.0) as u64)
    }
    ```
  - Update frontend `getDoomscrollDurationMs()` in `appStore.ts`:
    ```typescript
    function getDoomscrollDurationMs(level: number): number {
      const scrollMinutes = 1 + 4 * ((level - 1) / 9);  // 1→5 min linearly
      return scrollMinutes * 60 * 1000;
    }
    ```

  **Must NOT do**:
  - Use capping (formulas handle all levels naturally)

  **Acceptance Criteria**:
  - [ ] Level 1: 1 min scroll, 49 min standby (2% active)
  - [ ] Level 10: 5 min scroll, 0.5 min standby (91% active)
  - [ ] Formula is convex (standby drops faster at high levels)

  **Commit**: YES
  - Message: `fix(notifications): reduce level 10 standby to 30 seconds`
  - Files: `src-tauri/src/notifications/mod.rs`

- [ ] 2. Rust - set_consumption_threads command and buffer integration

  **What to do**:
  - Add `THREAD_COUNT` global atomic (like `THROTTLE_LEVEL`)
  - Create `set_consumption_threads(count: u8)` Tauri command
  - Implement buffer sizing: `target_buffer = 20 * thread_count`
  - Add `get_pending_count()` DB method if not exists
  - On thread count change: trigger crawler if buffer < target

  **Must NOT do**:
  - Change thread count mid-phase (only when standby)

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Straightforward command + buffer logic
  - **Skills**: [`rust`, `tauri`]
    - `rust`: DB operations and atomic globals
    - `tauri`: Command registration pattern

  **References**:
  - `src-tauri/src/commands.rs:18-25` - THROTTLE_LEVEL pattern
  - `src-tauri/src/db/mod.rs:287-295` - get_pending_count pattern
  - `src-tauri/src/crawler/scheduler.rs:59-86` - CrawlScheduler trigger

  **Acceptance Criteria**:
  - [ ] `set_consumption_threads(n)` command registered
  - [ ] THREAD_COUNT atomic stores value (clamped 1-8)
  - [ ] Buffer target = 20 × n
  - [ ] Crawler triggers when buffer < target
  - [ ] cargo check passes

  **Agent-Executed QA Scenarios**:

  \`\`\`
  Scenario: Thread count clamped to valid range
    Tool: Bash
    Preconditions: Tauri dev running
    Steps:
      1. invoke set_consumption_threads(0) → should clamp to 1
      2. invoke set_consumption_threads(10) → should clamp to 8
      3. invoke set_consumption_threads(4) → should be 4
    Expected Result: Values outside 1-8 are clamped
    Evidence: Command return values

  Scenario: Buffer target calculated correctly
    Tool: Bash
    Preconditions: DB with items
    Steps:
      1. Set threads to 2 → query buffer target → expect 40
      2. Set threads to 4 → query buffer target → expect 80
      3. Set threads to 8 → query buffer target → expect 160
    Expected Result: buffer_target = 20 × threads
    Evidence: DB query results
  \`\`\`

  **Commit**: YES
  - Message: `feat(consumption): add thread count command and buffer integration`
  - Files: `src-tauri/src/commands.rs`, `src-tauri/src/db/mod.rs`

- [ ] 3. Frontend - Thread slider UI component

  **What to do**:
  - Create vertical slider component (1-8 range)
  - Slider triggers `setConsumptionThreads(n)` on change
  - Add visual feedback showing selected thread count
  - Position: near existing intensity knob

  **Must NOT do**:
  - Change slider orientation (must be vertical)
  - Allow values outside 1-8

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
    - Reason: Custom vertical slider with React/Tailwind
  - **Skills**: [`react`, `tailwindcss`]
    - `react`: Component state and event handling
    - `tailwindcss`: Styling for vertical layout

  **References**:
  - `src/components/Knob.tsx` - Existing intensity control UI pattern
  - `src/stores/appStore.ts:201-205` - setThrottleLevel pattern

  **Acceptance Criteria**:
  - [ ] Vertical slider with 1-8 range
  - [ ] Slider visible in main UI
  - [ ] Value changes trigger `setConsumptionThreads` call
  - [ ] Slider disabled during doomscrolling phase

  **Agent-Executed QA Scenarios**:

  \`\`\`
  Scenario: Vertical slider controls thread count
    Tool: Playwright
    Preconditions: App running, slider visible
    Steps:
      1. Locate vertical slider element
      2. Set value to 4 using slider controls
      3. Verify displayed value is 4
    Expected Result: Slider shows 4, value stored in state
    Evidence: Screenshot of slider at value 4

  Scenario: Slider triggers backend command
    Tool: Playwright + Bash
    Preconditions: Slider at default, monitoring Rust logs
    Steps:
      1. Change slider to 3
      2. Check Rust logs for "set_consumption_threads(3)"
    Expected Result: Backend receives thread count change
    Evidence: Rust log output
  \`\`\`

  **Commit**: YES
  - Message: `feat(ui): add vertical thread slider component`
  - Files: `src/components/ThreadSlider.tsx`, `src/components/KnobControl.tsx`

- [ ] 4. Rust - Per-thread notification integration

  **What to do**:
  - Extend `ConsumptionThread` to fire notification on completion
  - Each thread: after `consumePendingItems`, call `send_thread_notification`
  - Notification includes: thread number, items consumed, time elapsed
  - Aggregate thread stats when all complete

  **Must NOT do**:
  - Send duplicate notifications
  - Block thread completion on notification

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Straightforward notification pattern extension
  - **Skills**: [`rust`, `tauri-notification`]
    - `rust`: Async notification triggering
    - `tauri-notification`: Existing notification plugin

  **References**:
  - `src-tauri/src/notifications/mod.rs:55-76` - send_teaser pattern
  - `src-tauri/src/commands.rs:58-62` - consumePendingItems call pattern

  **Acceptance Criteria**:
  - [ ] Each thread fires notification on completion
  - [ ] Notification shows thread number and consumption summary
  - [ ] All notifications fire before phase timeout

  **Agent-Executed QA Scenarios**:

  \`\`\`
  Scenario: Each thread fires notification on completion
    Tool: Playwright
    Preconditions: Threads=3, doomscrolling started
    Steps:
      1. Start doomscrolling phase
      2. Capture all notifications during phase
      3. Count unique thread notifications
    Expected Result: 3 notifications, one per thread
    Evidence: Screenshot of notifications

  Scenario: Notification includes thread info
    Tool: Playwright
    Preconditions: Thread notifications enabled
    Steps:
      1. Trigger single thread completion
      2. Read notification title and body
    Expected Result: Title="Thread 2 complete", body shows items consumed
    Evidence: Notification content screenshot
  \`\`\`

  **Commit**: YES
  - Message: `feat(notifications): add per-thread completion notifications`
  - Files: `src-tauri/src/consumption_threads.rs`, `src-tauri/src/notifications/mod.rs`

- [ ] 5. Frontend - Status bar thread count display

  **What to do**:
  - Add status indicator showing active thread count
  - Text: "Scrolling with N threads" when doomscrolling
  - Show thread icons or visual representation
  - Update in real-time as threads complete

  **Must NOT do**:
  - Hide during active scrolling
  - Show incorrect count

  **Recommended Agent Profile**:
  - **Category**: `frontend-ui-ux`
    - Reason: Status display with real-time updates
  - **Skills**: [`react`, `tailwindcss`, `zustand`]
    - `react`: Real-time state rendering
    - `zustand`: Store integration for thread count

  **References**:
  - `src/stores/appStore.ts:96-100` - systemStatus state pattern
  - `src/components/StatusBar.tsx` - Existing status display

  **Acceptance Criteria**:
  - [ ] Status shows "Scrolling with N threads" during doomscrolling
  - [ ] Count updates if threads change mid-phase (rare)
  - [ ] Visual distinction from other status states

  **Agent-Executed QA Scenarios**:

  \`\`\`
  Scenario: Status displays thread count during doomscrolling
    Tool: Playwright
    Preconditions: App running, slider set to 4
    Steps:
      1. Start doomscrolling phase
      2. Locate status display area
      3. Verify text shows "Scrolling with 4 threads"
    Expected Result: Status bar shows correct thread count
    Evidence: Screenshot of status bar

  Scenario: Status updates when threads change
    Tool: Playwright
    Preconditions: Doomscrolling active, threads=2
    Steps:
      1. Observe status showing 2 threads
      2. Wait for completion
      3. Verify status returns to standby
    Expected Result: Status transitions correctly
    Evidence: Before/after screenshots
  \`\`\`

  **Commit**: YES
  - Message: `feat(ui): add thread count to status display`
  - Files: `src/components/StatusBar.tsx`, `src/stores/appStore.ts`

- [ ] 6. Integration - Slider triggers crawler

  **What to do**:
  - Connect thread slider to crawler trigger
  - When slider changes: `setConsumptionThreads(n)` → check buffer → trigger crawl if needed
  - Debounce rapid slider changes
  - Visual indicator when crawler running

  **Must NOT do**:
  - Trigger crawler on every slider pixel drag
  - Block UI during crawler run

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
    - Reason: UI-state integration with backend
  - **Skills**: [`react`, `tauri`, `zustand`]
    - `react`: Event handling and debouncing
    - `tauri`: Invoke pattern for crawler trigger

  **References**:
  - `src/stores/appStore.ts:201-205` - setThrottleLevel pattern
  - `src/lib/tauri.ts` - invoke pattern for Tauri commands

  **Acceptance Criteria**:
  - [ ] Slider change triggers crawler if buffer < target
  - [ ] Debounce prevents excessive crawler triggers
  - [ ] UI shows "Crawling..." indicator when active

  **Agent-Executed QA Scenarios**:

  \`\`\`
  Scenario: Slider change triggers crawler
    Tool: Playwright + Bash
    Preconditions: Buffer has few items
    Steps:
      1. Set slider from 1 to 4
      2. Wait for crawler to complete
      3. Verify buffer now has 80 items (20×4)
    Expected Result: Buffer grows to target size
    Evidence: DB query showing 80 pending items

  Scenario: Rapid slider changes debounced
    Tool: Playwright
    Preconditions: Monitoring crawler triggers
    Steps:
      1. Rapidly change slider: 1→2→3→4→5
      2. Count crawler invocations
    Expected Result: Max 1-2 crawler triggers (debounced)
    Evidence: Crawler log timestamps
  \`\`\`

  **Commit**: YES
  - Message: `feat(integration): connect thread slider to crawler trigger`
  - Files: `src/stores/appStore.ts`, `src/components/ThreadSlider.tsx`

---

## Commit Strategy

| After Task | Message | Files | Verification |
|------------|---------|-------|--------------|
| 1 | `feat(consumption): add multithreaded consumption scheduler` | `consumption_threads.rs` | `cargo test --lib` |
| 2 | `feat(consumption): add thread count command and buffer integration` | `commands.rs`, `db/mod.rs` | `cargo check` |
| 3 | `feat(ui): add vertical thread slider component` | `ThreadSlider.tsx` | `npm run build` |
| 4 | `feat(notifications): add per-thread completion notifications` | `consumption_threads.rs`, `notifications/mod.rs` | cargo test |
| 5 | `feat(ui): add thread count to status display` | `StatusBar.tsx`, `appStore.ts` | npm run build |
| 6 | `feat(integration): connect thread slider to crawler trigger` | `appStore.ts`, `ThreadSlider.tsx` | E2E test |

---

## Success Criteria

### Verification Commands
```bash
# Rust tests
cargo test --lib consumption_threads  # All thread tests pass
cargo check                           # No compilation errors

# Frontend build
npm run build                         # Production build succeeds

# Integration
npm run tauri dev                     # Dev server starts
# Manual: Verify slider, status, notifications work
```

### Final Checklist
- [ ] Vertical slider (1-8) visible and functional
- [ ] Status shows "Scrolling with N threads"
- [ ] N parallel threads fire on doomscrolling start
- [ ] Thread durations: 1/2 to full budget, equally spaced
- [ ] Per-thread notifications fire
- [ ] Buffer: 20 × thread_count items maintained
- [ ] Slider change triggers crawler
- [ ] All tests pass
