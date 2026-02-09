# Flaky Tests

Tests that intermittently fail due to race conditions or timing dependencies.

## `wallet::resume::wallet_resume_by_rune_name`

**File:** `tests/wallet/resume.rs`

**Root cause:** Uses `thread::sleep(Duration::from_secs(1))` before sending
SIGINT to a subprocess. If the subprocess hasn't fully started within 1 second
(possible under heavy CPU load when all tests run in parallel), the signal
arrives too early and the test fails because the expected graceful shutdown
message is not produced.

**Related tests:**
- `wallet::resume::resume_suspended` (same pattern, same 1-second sleep at
  line 198)
- `wallet::resume::wallet_resume` (similar subprocess + signal pattern)
- `wallet::resume::wallet_resume_by_rune_not_found` (similar pattern)

**Suggested fix:** Replace the fixed sleep with a readiness check (e.g., poll
a health endpoint or wait for a specific log message on stderr) before sending
the signal.
