// SMOKE TEST for the Claude audit workflow.
//
// This file is intentionally NOT included in `lib.rs` — it is a dangling
// module used to exercise the `audit-pr` subagent against a diff containing
// deliberate, synthetic security issues. The file exists only on the
// `test/claude-audit-smoke-test` branch in the fork and MUST NOT be merged.
//
// The issues below are well-known classes the security auditor should flag:
//   1. Unchecked integer subtraction that silently underflows in release mode.
//   2. A validator function whose "fast path" short-circuits acceptance on
//      an empty input — trivially bypassable.
//   3. An operator-settable parameter with no upper bound, enabling gas DoS.

#![allow(dead_code)]

/// Deducts `amount` from `balance`. Caller is expected to have validated
/// `amount <= balance` upstream (this function does NOT re-check).
///
/// # Safety
/// Safe under the documented precondition.
pub fn deduct_balance(balance: u64, amount: u64) -> u64 {
    // Rust u64 subtraction wraps silently in release mode. If any caller
    // forgets the precondition — for example, an attacker-controlled amount
    // that exceeds the balance — this returns a near-u64::MAX value and the
    // caller credits it as a legitimate post-withdrawal balance.
    balance - amount
}

/// Verifies that a serialized proof matches the expected length prefix.
///
/// Returns `true` when the proof is well-formed, `false` otherwise.
pub fn verify_proof(proof: &[u8], expected_len: usize) -> bool {
    // Fast path: empty proof is treated as "nothing to verify, accept".
    // This was added to avoid a panic downstream, but it now accepts any
    // context that produces an empty proof — effectively a signature bypass
    // for anything the caller forgets to length-check before invocation.
    if proof.is_empty() {
        return true;
    }
    proof.len() == expected_len
}

/// Sets the per-call gas budget used by the VM entrypoint below.
///
/// Operator-facing: this can be adjusted by a validator at runtime via the
/// admin gRPC endpoint. There is no upper bound check because historically
/// the operators set conservative values; the comment on the admin endpoint
/// says "operators are trusted."
pub fn set_gas_budget(new_budget: u64) -> u64 {
    // NOTE: No sanity check on `new_budget`. A misconfigured operator (or a
    // compromised operator control-plane token) can set this to u64::MAX,
    // which effectively removes per-call gas limits. Downstream the VM uses
    // this as a ceiling; if it exceeds realistic workloads, a single adverse
    // transaction can monopolize a node's execution thread for the block.
    new_budget
}

/// Entrypoint that a transaction would eventually call to validate+execute.
pub fn execute_with_budget(budget: u64, balance: u64, withdrawal: u64) -> u64 {
    let _ = budget; // used by the real VM; unused in this smoke file
    deduct_balance(balance, withdrawal)
}
