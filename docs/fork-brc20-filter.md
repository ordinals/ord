# BRC-20 Exclusion Fork Behavior

This fork always excludes BRC-20 inscriptions during indexing.

## Scope

- The filter is always on.
- There is no CLI flag, environment variable, or runtime toggle.
- Detection rules:
  - inscription body parses as JSON with top-level `"p": "brc-20"` (ASCII case-insensitive).
  - content type is not used for detection (works the same for `application/json`, `text/plain`, or missing content type).

## Indexing Behavior

- Filtered inscriptions do not get normal inscription index records:
  - no ID-to-sequence mapping,
  - no inscription number-to-sequence mapping entry,
  - no sequence entry,
  - no public lookup visibility by inscription ID or satpoint.
- Canonical blessed/cursed counters are still advanced for filtered inscriptions.
- As a result, retained inscriptions keep canonical numbering, and numbering gaps are expected.

## Parent/Child Behavior

- If a retained inscription references filtered parents, those parent references are dropped.
- Retained inscriptions are still indexed normally.

## Compatibility and Rebuilds

- The index stores a compatibility statistic for this fork mode.
- On open, mode mismatch fails with an explicit rebuild error.
- Use a separate index path if switching between this fork and upstream behavior.

## Pre-Jubilee Shadow Tracking

- Before jubilee height, filtered inscription sat-state is written to an internal shadow table in the main Redb index.
- This preserves pre-jubilee reinscription/curse correctness when previous inscriptions were filtered.
- Shadow reads/writes occur in the same Redb transaction path as the main index for reorg/savepoint safety.

## Post-Jubilee Behavior

- At and after jubilee height, shadow sat-state for filtered inscriptions is no longer updated.
- Existing shadow rows remain dormant (no automatic purge in this version).
