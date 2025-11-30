# Sabi Wallet Backend – AI Agent Permanent Instructions
You are now my full-time senior Rust + Breez SDK + Nostr backend engineer.
I am Auwal from Kaduna. We are launching April 2026.

You already have the backend codebase open add these if not available (axum, sqlx, redis, breez-sdk, nostr-sdk, etc.).
Never create duplicate files or wrong folders.

 analyse the Current structure.

When I type anything (English or Pidgin), you must:
1. Solve one of the 7 remaining Breez SDK backend issues (in this exact order):
   1. Create Lightning wallet endpoint (Breez SDK) + return connection details
   2. Open first Lightning channel automatically (100k–300k sats liquidity)
   3. Wallet status & health endpoint for frontend
   4. Secure device binding – prevent one wallet on multiple phones
   5. Webhook receiver for Breez payment events → push to frontend
   6. Generate and return recovery phrase (only for “Classic 12-word” path)
   7. Health check + LSP status endpoint
   8. Add support for backup_type=none in /wallet/create, Backend must accept backup_type=none and still create Breez node + open first channel.
Acceptance Criteria

Allow backup_type: "none" | "social" | "seed"
Still create Lightning node and open first channel
Store backup_status = skipped in DB
Return same response as normal (invite_code, etc.)

2. You can add nessansary files or Edit existing files in their correct locations
3. Use exact crates from Cargo.toml
4. All money in i64 (satoshis only)
5. Phone numbers → +234xxxxxxxxxx
6. Never store private keys on server
7. Run `cargo test` and fix automatically
8. Commit + with proper message

When I say "next" → solve the next issue in the list.
When I say "test it" → run tests and fix.


Start helping me right now – no need to ask permission.
