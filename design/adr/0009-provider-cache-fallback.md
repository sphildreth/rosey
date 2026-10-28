# ADR-0009: Keep provider lookups available when the disk cache fails

Status: Accepted
Date: 2026-10-27

## Context

Identify could fail before searching providers when its DecentDB cache could not open.
An existing format-11 cache with a format-14 build reproduced this when
`decentdb-migrate` was unavailable. The CLI and TUI discarded the underlying
error, leaving only "Failed to initialize metadata provider cache".

Provider metadata is reusable API data; its persistence should not be required
for identification.

## Decision

- Keep `ProviderCache::open` strict, including the existing migration and explicit
  reset behavior.
- Have `ProviderManager::new` try the disk cache first, then create an independent
  in-memory DecentDB cache if opening or initializing the disk cache fails.
- Log the cache path, database error, and its source when falling back. Preserve
  error details in CLI/TUI identification if even the session cache fails.
- Use the same schema and TTL for both cache modes. All provider lookups continue
  through the existing manager API.
- Do not add automatic deletion or reset of an incompatible cache. A missing
  migration tool leaves the original database and WAL available for recovery.

## Consequences

- Manual Identify and automatic batch identification keep working with an
  incompatible, corrupt, or unwritable disk cache.
- Cached results are lost when a fallback manager is dropped, so later sessions
  can make more API requests until disk caching is restored.
- Existing healthy caches remain persistent; cache failures are visible in logs.

## Validation

Temp-directory tests cover an unsupported database with a missing migrator,
preservation of its database and WAL, provider lookups through a session cache,
an unusable cache directory, and persistence across healthy manager instances.
