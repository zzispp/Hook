# Progress Log

> Auto-maintained by Taskmaster. Each entry records what happened, why, and what's next.
> This file serves as both decision audit trail and context-recovery anchor.

---

## Session Start

- **Date**: YYYY-MM-DD HH:MM
- **Task name**: `<task-name>`
- **Task dir**: `.codex-tasks/<task-name>/`
- **Spec**: See SPEC.md
- **Plan**: See TODO.csv (N milestones)
- **Environment**: <language> / <framework> / <test runner>

---

## Context Recovery Block

> If you are resuming this task after compaction, session restart, or context loss,
> read this section FIRST to restore working state.

- **Current milestone**: #N — <title>
- **Current status**: IN_PROGRESS | WAITING_SUBTASK | WAITING_BATCH | BLOCKED_EXTERNAL | BLOCKED_SYSTEM
- **Last completed**: #N-1 — <title>
- **Current artifact**: `<TODO.csv | SUBTASKS.csv | batch/workers-output.csv | <path>>`
- **Key context**: <1-2 sentences summarizing where we left off>
- **Known issues**: <any unresolved problems>
- **Next action**: <exact next step to take>

> Update this block EVERY TIME a milestone changes status.

---

<!-- Append entries below as each milestone completes -->

## Milestone N: <title>

- **Status**: DONE | FAILED
- **Started**: HH:MM
- **Completed**: HH:MM
- **What was done**:
  -
- **Key decisions**:
  - Decision: ...
  - Reasoning: ...
  - Alternatives considered: ...
- **Problems encountered**:
  - Problem: ...
  - Resolution: ...
  - Retry count: 0
- **Validation**: `<command>` → exit 0 / exit 1 + error
- **Files changed**:
  - `path/to/file` — <what changed>
- **Next step**: Milestone N+1 — <title>

---

<!-- Final summary goes here when all milestones are DONE -->

## Final Summary

- **Total milestones**: X
- **Completed**: X
- **Failed + recovered**: X
- **External unblock events**: X
- **Total retries**: X
- **Files created**: X
- **Files modified**: X
- **Key learnings**:
  -
- **Recommendations for future tasks**:
  -
