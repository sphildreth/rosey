# ADR-0007: Automatic provider identification and review queue

Status: Accepted
Date: 2026-10-18

## Context

Identification in Rosey was automatic only for offline parsing (filenames, folders, NFO files) and for confirming path-embedded `[tmdbid-*]` tags. Everything else relied on a per-file manual identify overlay: the user edited kind/title/year, pressed `F5` to search providers, and picked a result one file at a time.

This made large libraries painful:

- TV shows were never provider-enriched automatically; `search_tv_all` and `get_episode` existed but were unreachable from automatic flows.
- Plan and run flows silently dropped below-threshold items, so identification failures vanished instead of being queued for follow-up.
- Batch identification re-opened the provider cache and rebuilt provider clients for every file.

The Python reference has the same manual-first identify dialog, so changing this is an intentional deviation from parity.

## Decision

1. **Automatic TV show confirmation.** When online providers are configured, offline-identified episodes/shows without an existing `tmdbid`/`tvdbid` are auto-confirmed through `search_tv_all`:
   - a single provider result is selected automatically;
   - with multiple results, selection requires an exact normalized title match plus a compatible year when the parsed year is known;
   - when no year is parsed, only an unambiguous single result is selected, because a wrong show pick would organize episodes into the wrong series.
2. **Review queue instead of silent drops.** The TUI plan flow and CLI `rosey run` keep below-threshold items instead of discarding them. TUI items carry `needs_review` and are excluded from moves and destination-conflict checks; the Plan Preview marks them (`~` prefix, `n` jumps to the next one). CLI JSON output gains a `needs_review` array and text output lists them.
3. **Apply-and-next identify loop.** The identify overlay opens with an automatic provider search, and applying an identification continues to the next unresolved scanned file until none remain. `Esc` exits the loop.
4. **Batch identification plumbing.** `rosey_metadata::BatchIdentifier` builds one provider manager (one cache handle, one client set) for a whole batch; `identify_file_with_metadata` is now a one-shot wrapper over it. CLI `run`/`identify` and the TUI plan worker use the batch form.

## Consequences

Positive:

- Provider-configured libraries identify movies and TV shows without per-file manual work.
- Identification failures are visible and actionable instead of hidden.
- Batch runs open the cache database once, cutting per-file overhead.

Negative:

- Automatic show selection can mis-pick when providers return ambiguous results; the year and single-result guards reduce but do not eliminate this.
- More provider requests during bulk planning (rate limiter and cache absorb this; first unseeded runs are slower).
- `IdentifiedItem` gains a state field, so serialized TUI snapshots gain a key.

## Alternatives considered

- Keep title search manual-only (Python parity): rejected; the manual loop was the dominant cost for large libraries.
- Auto-select any top search result: rejected; wrong-show auto-organization is worse than asking.
- Drop still-below-threshold items as before: rejected; silent failure hides exactly the files that need attention.
- Fetch per-episode titles via `get_episode` automatically: deferred; show-level confirmation is the correctness-critical step, per-episode lookups multiply requests and can wait for a follow-up slice.

## Follow-up

- Recalibrate confidence scoring so parse-only identifications can reach yellow (see ADR-0008).
- Consider per-episode title enrichment behind a config flag.
