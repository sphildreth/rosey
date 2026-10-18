# ADR-0008: Recalibrate confidence scoring for parse-only identification

Status: Proposed
Date: 2026-10-18

## Context

The identification scorer (`rosey-core::scorer`) was calibrated for ID-backed identification: NFO or provider IDs contribute 40-50 points, and parsed title/year/season add at most 30-35. With default thresholds (green 70, yellow 40):

- an offline-parsed movie with title and year scores ~25 and can never pass yellow;
- a fully parsed episode with title, season/episode, and episode title scores exactly 40.

The consequence is that the automatic plan/run pipeline keeps almost nothing from parsing alone unless NFO IDs or online providers are present, which pushes users into manual identification even for cleanly named files. This blocks the goal of making identification more automatic (ADR-0007).

Changing the scorer changes documented Python-parity behavior and interacts with user-configured thresholds, so it needs an explicit decision before implementation.

## Decision (proposed)

Split the scorer's notion of confidence into two tiers:

- **Parse tier:** a cleanly parsed title plus year (movies) or title plus season/episode or date (episodes) should reach yellow without any external ID.
- **Confirmation tier:** NFO/path/provider IDs, provider-confirmed titles, and duration/directory checks earn green.

Concretely (to be tuned with tests): raise parse-derived points and/or introduce band-aware scoring so that parse-only results land in yellow, while keeping ID-backed results comfortably green. Threshold config keys and their meanings do not change.

## Consequences

Positive:

- Offline-first users get an automatic pipeline that keeps well-named files without providers.
- Confidence bands align with user expectations: "looks right" is yellow, "confirmed" is green.

Negative:

- Intentional deviation from Python scorer outputs; golden parity tests for scores must be updated with a documented mapping.
- Users who tuned thresholds against current scores may need to revisit them; a config note in the release notes is required.
- More borderline items will pass yellow, increasing reliance on review-queue signals rather than the threshold alone.

## Alternatives considered

- Keep scores and lower default thresholds: rejected; it would also lower the bar for genuinely weak identifications (unknown kind, missing titles).
- Add a separate "auto-move gate" distinct from confidence: rejected for now; two overlapping thresholds confuse users and config.
- Do nothing and require providers: rejected; offline-first is a PRD non-goal violation for the default experience.

## Follow-up

- Implement behind tests with a before/after score table in `design/migration/`.
- Update golden parity expectations and record the deviation in the parity notes.
