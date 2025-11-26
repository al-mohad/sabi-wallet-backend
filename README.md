# sabi-wallet-backend ⚡₦
**Backend for Sabi Wallet – The Moniepoint of Bitcoin**  
Built in Kaduna, written in Rust, made for Nigeria & Africa.

This repo powers the server side of Sabi Wallet:
- USSD full wallet access (*333*777#) – works on Nokia 3310  
- Fiat on/off-ramp webhook (GTBank/Paystack → instant BTC delivery)  
- Buy & Sell Bitcoin engine (Bitnob-style)  
- Social recovery coordinator (3-of-5 Shamir shares over encrypted Nostr DMs)  
- Nostr relay + key management  
- Admin dashboard & moderation queue  
- Rate limiting & fraud detection (Nigerian traffic patterns)

### Tech Stack (2025 battle-tested)
- **Rust** + **axum** + **tower**
- **Breez SDK** (Lightning payments & liquidity)
- **nostr-sdk** (full client + encrypted DMs)
- **PostgreSQL** + **sqlx**
- **Redis** (sessions, rate limiting, trade state)
- **Africa’s Talking** (USSD gateway)
- Shamir secret sharing (recovery shares)
- Paystack webhook integration
- Docker + docker-compose ready

### Features already working (Dec 2025)
- [x] USSD menu (balance, send, receive, buy BTC)
- [x] Paystack → Breez instant BTC delivery
- [x] Social recovery request & share submission
- [x] Buy/Sell Naira ↔ BTC (we act as counterparty)
- [x] Admin login + manual release dashboard
- [x] Rate limiting per phone number

### Local development
```bash
cp .env.example .env
docker compose up -d postgres redis
cargo run
# USSD simulator: http://localhost:8000/ussd
# Admin dashboard: http://localhost:8000/admin