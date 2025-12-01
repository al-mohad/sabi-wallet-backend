# Sabi Wallet Backend - Codebase Analysis

**Date:** December 1, 2025  
**Version:** 0.1.0  
**Analyzed by:** AI Agent (Copilot)

## Executive Summary

Sabi Wallet Backend is a Rust-based Bitcoin/Lightning wallet backend designed for Nigerian and African markets. The project provides USSD access, Lightning Network support via Breez SDK, Nostr integration, and social recovery features.

**Current Status:**
- ✅ Basic project structure established
- ✅ Database schema designed
- ⚠️ Build issues with `lettre` crate configuration
- ⚠️ Mock implementations in place (not production-ready)
- ❌ Missing 8 core Breez SDK features

---

## 1. Project Architecture

### Technology Stack

| Component | Technology | Version | Status |
|-----------|-----------|---------|--------|
| Language | Rust | 1.91.1 | ✅ Active |
| Web Framework | axum | 0.7 | ✅ Configured |
| Middleware | tower, tower-http | 0.4/0.5 | ✅ Configured |
| Database | PostgreSQL via sqlx | 0.7 | ✅ Configured |
| Cache | Redis | 0.25 | ✅ Configured |
| Async Runtime | tokio | 1.x | ✅ Configured |
| Logging | tracing, sentry | 0.1/0.34 | ✅ Configured |
| Bitcoin | bitcoin crate | 0.31 | ✅ Configured |
| Lightning | Breez SDK | N/A | ❌ Mock only |
| Nostr | nostr-sdk | 0.27 | ✅ Configured |
| Crypto | Shamir, argon2 | 2.0/2.0 | ✅ Configured |
| Email | lettre | 0.11 | ⚠️ Config error |

### Module Structure

```
src/
├── main.rs                 # Application entry point
├── app_state.rs           # Shared application state
├── config.rs              # Environment configuration
├── database.rs            # Database connection pooling
├── error.rs               # Centralized error handling
├── routes.rs              # Route registration
├── cli.rs                 # Command-line interface
│
├── api/                   # HTTP handlers
│   ├── admin.rs          # Admin dashboard endpoints
│   ├── recovery.rs       # Social recovery endpoints
│   ├── ussd.rs           # USSD callback handlers
│   ├── wallet.rs         # Wallet creation/management
│   └── webhooks.rs       # Webhook receivers (Paystack, Breez)
│
├── services/             # Business logic layer
│   ├── admin_service.rs
│   ├── fiat_service.rs
│   ├── nostr_service.rs
│   ├── recovery_service.rs
│   ├── ussd_service.rs
│   └── wallet_service.rs
│
├── domain/               # Domain models and types
│   ├── models.rs        # Database entity models
│   └── types.rs         # Custom types (Sats, Kobo)
│
├── bitcoin/             # Bitcoin/Lightning logic
│   └── breez.rs        # Breez SDK integration (mock)
│
├── nostr/               # Nostr protocol integration
│   ├── client.rs       # Nostr client
│   └── relay.rs        # Relay management
│
└── utils/               # Utility functions
    └── phone_number.rs  # Phone number validation
```

---

## 2. Database Schema

### Tables

1. **users**
   - Stores user accounts (phone number based)
   - Columns: id, phone_number, created_at, updated_at
   - Index: phone_number (unique)

2. **wallets**
   - Lightning wallet metadata
   - Columns: id, user_id, nostr_npub, breez_wallet_id, balance_sats, backup_type, backup_status
   - Support for backup_type: 'none' | 'social' | 'seed'
   - Index: user_id, nostr_npub, breez_wallet_id

3. **encrypted_nostr_nsecs**
   - Encrypted Nostr private keys for recovery coordinator
   - Columns: id, wallet_id, encrypted_nsec, created_at

4. **transactions**
   - Transaction history
   - Columns: id, wallet_id, tx_type, amount_sats, fee_sats, status, description, external_id
   - Index: wallet_id, external_id

5. **admin_users**
   - Admin authentication
   - Columns: id, username, password_hash, is_active

6. **fiat_onramp_webhooks**
   - Webhook event log from payment providers
   - Columns: id, provider, event_id, payload, processed
   - Index: event_id, processed

### Migration Status
- ✅ Initial schema created: `20251126120000_initial_setup.sql`
- ✅ Automated triggers for `updated_at` columns
- ✅ Proper indexing for common queries

---

## 3. API Endpoints

### Implemented Endpoints

| Method | Path | Handler | Status |
|--------|------|---------|--------|
| GET | /health | Health check | ✅ Basic |
| POST | /api/wallet/create | Create Lightning wallet | ✅ Mock |
| GET | /api/wallet/:user_id | Get wallet info | ✅ Mock |
| POST | /api/recovery/request | Request recovery | 🔄 Stub |
| POST | /api/recovery/submit | Submit share | 🔄 Stub |
| POST | /api/ussd/ | USSD callback | 🔄 Stub |
| POST | /api/webhook/breez | Breez webhook | 🔄 Stub |
| POST | /api/webhook/paystack | Paystack webhook | 🔄 Stub |
| POST | /api/admin/login | Admin login | 🔄 Stub |
| GET | /api/admin/trades | Get trades | 🔄 Stub |
| POST | /api/admin/manual-release | Manual release | 🔄 Stub |
| GET | /api/rates | Exchange rates | 🔄 Stub |
| GET | /api/health/breez | Breez health | ❌ Not implemented |

### Missing Critical Endpoints (Per Issue Tracker)

1. **Wallet Status** (#9) - `GET /api/v1/wallet/status`
2. **Device Binding** (#11) - Middleware for device validation
3. **Event Stream** (#12) - `GET /events` (SSE)
4. **Recovery Contact Resolution** (#2) - `POST /api/v1/recovery/resolve-contacts`
5. **Recovery Status** (#5) - `GET /api/v1/recovery/status/:npub`
6. **Health + LSP Status** (#15) - `GET /health` (enhanced)

---

## 4. Configuration Management

### Environment Variables (from `config.rs`)

**Required:**
- `APP_SECRET_KEY` - Application encryption key
- `DATABASE_URL` - PostgreSQL connection string
- `REDIS_URL` - Redis connection string
- `SABI_NOSTR_NSEC` - Server's Nostr private key
- `BREEZ_API_KEY` - Breez SDK API key
- `BREEZ_MNEMONIC` - Breez hot wallet seed
- `BREEZ_ENVIRONMENT` - Breez environment (mainnet/testnet)
- `PAYSTACK_SECRET_KEY` - Paystack API key
- `AT_API_KEY` - Africa's Talking API key
- `AT_USERNAME` - Africa's Talking username
- `DEFAULT_ADMIN_PASSWORD` - Initial admin password

**Optional:**
- `APP_ENV` - Environment (default: "dev")
- `SERVER_HOST` - Listen address (default: "0.0.0.0")
- `SERVER_PORT` - Listen port (default: 8080)
- `SENTRY_DSN` - Sentry error tracking
- `NOSTR_RELAYS` - Comma-separated relay URLs
- `SMTP_*` - Email configuration

**Security Features:**
- ✅ Secrets wrapped in `SecretString` from `secrecy` crate
- ✅ Secrets never logged or serialized unencrypted
- ✅ Sentry integration for production error tracking

---

## 5. Build Issues

### Current Build Error

**Issue:** `lettre` crate feature flag misconfiguration

```
error: feature `rustls` also requires either the `aws-lc-rs` or the `ring` feature
error: feature `rustls` requires `rustls-platform-verifier`, `rustls-native-certs` or `webpki-roots`
error: `tokio1` + `native-tls` require `tokio1-native-tls` feature
```

**Root Cause:**  
In `Cargo.toml`, lettre is configured as:
```toml
lettre = { version = "0.11", features = ["builder", "smtp-transport", "tokio1-rustls"] }
```

But `tokio1-rustls` requires additional crypto backend features.

**Solution:**  
Update to:
```toml
lettre = { version = "0.11", features = ["builder", "smtp-transport", "tokio1-rustls"], default-features = false }
```
And add one of:
- `ring` + `rustls-native-certs` OR
- `aws-lc-rs` + `webpki-roots`

**Recommendation:** Use `ring` + `rustls-native-certs` for better compatibility.

---

## 6. Missing Features (From GitHub Issues)

### Priority 1: MVP Blockers

1. **#8 - Create Lightning Wallet Endpoint** (HIGH PRIORITY, MVP, Lightning)
   - Real Breez SDK integration (currently mock)
   - Generate BIP-39 mnemonic
   - Derive xprv → Breez cloud node
   - Return invite_code, node_id, credentials
   - Open 100k-300k sats liquidity channel
   - Idempotent

2. **#10 - Open First Lightning Channel** (MVP, Lightning)
   - Auto-open 0-conf channel after wallet creation
   - Use Breez SDK `open_channel` or Greenlight API
   - Minimum 100k sats inbound
   - Retry logic for LSP delays
   - Track transaction for accounting

3. **#9 - Wallet Status Endpoint** (Lightning)
   - `GET /api/v1/wallet/status`
   - Return: onchain_balance, lightning_balance, inbound_liquidity, channels_count, node_online
   - Redis caching (15 seconds)
   - Auth via npub + signed Nostr event

### Priority 2: Security & Recovery

4. **#11 - Device Binding** (Security)
   - Generate device_id (UUID) on first wallet creation
   - Validate device_id header on all requests
   - Require signed Nostr event from wallet npub
   - Force recovery flow if device_id changes

5. **#1 - Social Recovery Setup** (Recovery, HIGH PRIORITY)
   - Integrate with wallet creation
   - Shamir 3-of-5 split on master seed
   - Encrypt shares with recipient npubs
   - Send via encrypted Nostr DMs
   - Zero-knowledge server (no recoverable data stored)

6. **#4 - Recovery Share Submission** (Recovery)
   - `POST /api/v1/recovery/submit-share`
   - Accept encrypted share + helper npub
   - Auto-reconstruct seed when 3/5 shares received
   - Return to mobile app securely

### Priority 3: UX & Monitoring

7. **#12 - Breez Webhook Receiver** (Lightning)
   - `POST /webhook/breez` with signature verification
   - Parse payment events, channel events, LSP refunds
   - Store in DB + push via SSE or Firebase
   - Frontend SSE stream: `GET /events`

8. **#14 - Generate Recovery Phrase** (Recovery)
   - `backup_type=seed` support
   - Generate 24-word BIP-39 phrase
   - Encrypt with device public key
   - Return exactly once
   - Never store server-side

9. **#15 - Health + LSP Status Endpoint** (Monitoring)
   - `GET /health`
   - Return: breez_node_online, lsp_available, pending_webhooks_count
   - 200 OK when healthy

### Priority 4: Nice-to-Have

10. **#2 - Contact Resolution** (Nostr)
    - `POST /api/v1/recovery/resolve-contacts`
    - Phone numbers → npub lookup
    - Query nostr.directory or our relay
    - Return npub, name, avatar
    - Flag for invite flow if no npub

11. **#3 - One-tap Nostr Key Generation** (Nostr)
    - Generate ed25519 keypair for contacts without npub
    - Create signed invitation event
    - Shorten to sabi.ng/invite/abc123
    - Update recovery share encryption after claim

12. **#5 - Recovery Health Dashboard** (Admin)
    - `GET /api/v1/recovery/status/:npub`
    - Show shares received, last seen, online/offline
    - Admin panel for pending recoveries

13. **#6 - Test Recovery Flow** (Testing)
    - `test=true` flag for recovery endpoints
    - Fake shares for testing
    - No real seed reconstruction

14. **#7 - Hausa + Pidgin Localization** (i18n)
    - Translate all recovery messages
    - Support for DMs, emails, USSD

---

## 7. Code Quality Assessment

### Strengths ✅

1. **Type Safety**
   - Custom types (`Sats`, `Kobo`) prevent unit confusion
   - Strong UUID usage for IDs
   - Proper error handling with custom `AppError` enum

2. **Security Practices**
   - Secrets wrapped in `SecretString`
   - Password hashing with Argon2
   - Prepared statements (sqlx) prevent SQL injection
   - CORS configured

3. **Observability**
   - Structured JSON logging
   - Request ID tracking
   - Sentry integration
   - Tracing middleware

4. **Database Design**
   - Proper foreign key constraints
   - Automated timestamps with triggers
   - Appropriate indexing
   - Support for multiple backup types

### Weaknesses ⚠️

1. **Mock Implementations**
   - Breez SDK integration is mocked
   - Wallet creation generates fake node IDs
   - No actual Lightning channel operations

2. **Missing Validation**
   - Phone number validation is basic (just prefix check)
   - No rate limiting middleware
   - Missing Nostr event signature validation

3. **Incomplete Error Handling**
   - Some endpoints return stub responses
   - No comprehensive error logging in services

4. **Testing**
   - No unit tests in services
   - No integration tests
   - Only basic type conversion tests in domain/types.rs

5. **Documentation**
   - Missing API documentation (OpenAPI/Swagger)
   - No inline examples
   - Sparse comments in complex logic

---

## 8. Security Considerations

### Current Measures ✅

- Argon2 for password hashing
- SecretString for sensitive config
- SQL injection prevention (sqlx prepared statements)
- CORS middleware
- Request ID tracking
- Encrypted Nostr nsecs in DB

### Needed Improvements ⚠️

1. **Authentication & Authorization**
   - Implement JWT or session-based auth
   - Nostr signature validation for wallet endpoints
   - Device binding enforcement

2. **Rate Limiting**
   - Per-phone-number limits (mentioned in README)
   - API endpoint rate limiting
   - Redis-based implementation needed

3. **Input Validation**
   - Strengthen phone number validation
   - Validate backup_type enum strictly
   - Sanitize all user inputs

4. **Secrets Management**
   - Rotate Breez mnemonic securely
   - Implement key rotation for app_secret_key
   - Use hardware security module (HSM) for production

5. **Audit Logging**
   - Log all admin actions
   - Track wallet creations and recovery attempts
   - Retention policy for compliance

---

## 9. Deployment Readiness

### Infrastructure

- ✅ Dockerfile present
- ✅ docker-compose.yml with postgres + redis
- ⚠️ No Kubernetes manifests
- ⚠️ No CI/CD pipeline configuration

### Environment Support

- ✅ Development environment configured
- ⚠️ Production hardening needed
- ❌ Staging environment not defined

### Monitoring & Alerts

- ✅ Sentry for error tracking
- ⚠️ No metrics collection (Prometheus?)
- ❌ No uptime monitoring
- ❌ No alerting rules

---

## 10. Dependencies Analysis

### Production Dependencies (49 total)

**Web Framework:**
- axum 0.7, tower 0.4, hyper 1.x - Modern, async-first

**Database:**
- sqlx 0.7 - Compile-time SQL verification, good choice
- redis 0.25 - Standard Rust Redis client

**Crypto:**
- bitcoin 0.31 - Up-to-date Bitcoin library
- nostr-sdk 0.27 - Nostr protocol support
- shamir 2.0 - Secret sharing
- rust-argon2 2.0 - Password hashing

**Utilities:**
- chrono 0.4 - Date/time handling
- uuid 1.6 - UUID generation
- serde, serde_json 1.0 - Serialization
- anyhow, thiserror 1.0 - Error handling

**External APIs:**
- reqwest 0.12 - HTTP client
- lettre 0.11 - Email (⚠️ config issue)

### Dev Dependencies

- rstest 0.18 - Parameterized testing
- mockall 0.12 - Mocking framework
- fake 2.9 - Test data generation
- wiremock 0.6 - HTTP mocking

**Assessment:** Good selection of mature, well-maintained crates. No major version conflicts.

---

## 11. Performance Considerations

### Current Optimizations ✅

1. **Connection Pooling**
   - Database pool via sqlx
   - Redis client configured

2. **Caching Strategy**
   - Wallet status cached for 15 seconds (design)
   - Redis for session/rate limiting state

3. **Async Architecture**
   - Full tokio async runtime
   - Non-blocking I/O throughout

### Potential Bottlenecks ⚠️

1. **Database Queries**
   - No query optimization analysis yet
   - Missing indexes for some common queries
   - No read replicas for scaling

2. **External API Calls**
   - Synchronous Breez SDK calls could block
   - No circuit breaker for external services
   - No request timeouts configured

3. **Webhook Processing**
   - Webhook handlers not optimized
   - No queue for async processing
   - Could block under high load

### Scaling Recommendations

1. Add message queue (RabbitMQ/Kafka) for webhook processing
2. Implement circuit breaker pattern for external APIs
3. Add database read replicas
4. Consider horizontal scaling with load balancer
5. Profile and optimize hot paths

---

## 12. Next Steps & Recommendations

### Immediate Actions (This Sprint)

1. **Fix Build Error** - Update lettre dependency configuration
2. **Run Tests** - Ensure existing tests pass
3. **Implement Issue #8** - Real Breez SDK wallet creation (Priority 1)
4. **Implement Issue #10** - Auto-open Lightning channel (Priority 1)

### Short Term (Next 2-4 Weeks)

1. **Complete MVP Features**
   - #9 Wallet status endpoint
   - #11 Device binding
   - #1 Social recovery setup

2. **Add Testing**
   - Unit tests for all services
   - Integration tests for critical paths
   - Load testing for USSD endpoints

3. **Security Hardening**
   - Implement rate limiting
   - Add authentication middleware
   - Validate Nostr signatures

### Medium Term (1-2 Months)

1. **Complete Recovery System**
   - #4 Share submission
   - #14 Seed phrase generation
   - #2 Contact resolution

2. **Monitoring & Operations**
   - #15 Health check endpoint
   - Prometheus metrics
   - Alert rules

3. **Infrastructure**
   - CI/CD pipeline
   - Staging environment
   - Production deployment guide

### Long Term (3-6 Months)

1. **Feature Completion**
   - All 15 GitHub issues resolved
   - USSD full implementation
   - Admin dashboard
   - Webhook event streaming

2. **Optimization**
   - Performance profiling
   - Database query optimization
   - Caching strategy refinement

3. **Compliance & Security**
   - Security audit
   - Penetration testing
   - Nigerian financial regulations compliance

---

## 13. Risk Assessment

| Risk | Severity | Likelihood | Mitigation |
|------|----------|-----------|------------|
| Breez SDK integration complexity | High | Medium | Thorough testing, fallback plans |
| LSP downtime | High | Low | Health monitoring, multiple LSP support |
| Private key exposure | Critical | Low | Zero-knowledge architecture, encryption |
| USSD gateway reliability | High | Medium | Retry logic, queue-based processing |
| Scalability limits | Medium | Medium | Horizontal scaling, caching |
| Regulatory compliance | High | Medium | Legal consultation, audit trail |
| Social recovery abuse | Medium | Low | Rate limiting, fraud detection |

---

## 14. Conclusion

Sabi Wallet Backend has a **solid architectural foundation** with:
- Clean modular structure
- Secure-by-default configuration
- Comprehensive database schema
- Good dependency selection

**However**, the project is in **early development stage** with:
- Mock implementations for critical features
- 8 high-priority features missing
- Build configuration issues
- No test coverage

**Recommended Focus:**
1. Fix lettre build error immediately
2. Implement real Breez SDK integration (#8, #10)
3. Add comprehensive testing
4. Complete security features (#11)
5. Deploy MVP with monitoring

**Estimated Timeline to MVP:** 6-8 weeks with dedicated development

---

**Analysis prepared by:** AI Agent (GitHub Copilot)  
**For:** Sabi Wallet Engineering Team  
**Next Review:** After MVP feature completion
