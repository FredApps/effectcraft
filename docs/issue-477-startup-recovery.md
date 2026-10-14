# Issue 477: startup adapter recovery

Tracking: https://github.com/storytold/effectcraft/issues/477

Status: investigation placeholder. No production behavior has changed, and the reported recurrence has not yet been reproduced on current main.

## Scope

Investigate the startup failure reporting that no suitable graphics adapter was found, with Vulkan not requested and the OpenGL backend incompatible with the window surface. Review the existing 0.7.0 launch-guard fix (384c8cc0943236c4590034b069efb8d111895d98) before changing behavior. Keep this separate from GPU compositing failures after a window has opened.

## Planned validation

- Trace Automatic and explicitly selected graphics backends, environment overrides, persisted preferences, and launch-pending recovery.
- Exercise stale and live launch markers, failed OpenGL initialization, and the next launch after failure.
- Reproduce the reported Linux/NVIDIA surface combination where the available test environment permits; document any hardware gap.
- Add a focused failing regression before a production fix, preserve explicit user choices, and avoid silent permanent backend changes.
- Run the affected tests and required repository checks; record only checks actually executed.

## Current evidence

The original report is closed, with a maintainer comment identifying a fix in 0.7.0. A matching error screenshot prompted this follow-up. The screenshot alone does not establish the application version or prove that the same cause remains in current main.

## Tests

Not run for this planning-only placeholder. Implementation and validation remain pending.
