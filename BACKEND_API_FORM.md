# Sabi Wallet Backend API Form

## For Flutter Frontend Integration

---

## 1. BACKEND BASE URL

**Development:** `http://localhost:8080`

**Production example:** `https://api.sabi.money`

*Use whichever matches your deployed backend.*

---

## 2. WALLET CREATION ENDPOINT

**Method:** POST

**Path:** `/wallet/create`

### Request Body (JSON)

```json
{
  "user_id": "00000000-0000-0000-0000-000000000000",
  "phone_number": "+2348012345678",
  "backup_type": "none"
}
```

**Field Notes:**
- `user_id`: UUID string (required, must be valid UUID format)
- `phone_number`: E.164 phone number (required, must start with `+` and be at least 10 digits)
- `backup_type`: Backup type (optional, defaults to "none"). Valid values: "none", "social", or "seed"

### Response Body (Success - HTTP 201 Created)

```json
{
  "success": true,
  "data": {
    "id": "11111111-1111-1111-1111-111111111111",
    "user_id": "00000000-0000-0000-0000-000000000000",
    "breez_wallet_id": "breez_111111111111111111111111111",
    "nostr_npub": "npub1....",
    "balance_sats": 0,
    "backup_type": "none",
    "backup_status": "skipped",
    "connection_details": {
      "wallet_id": "11111111-1111-1111-1111-111111111111",
      "user_id": "00000000-0000-0000-0000-000000000000",
      "lightning_node_id": "node_abc123...",
      "node_address": "lnd_node_abc123@127.0.0.1:9735",
      "synced": false,
      "initialized_at": "2025-11-30T12:34:56+00:00"
    },
    "created_at": "2025-11-30T12:34:56+00:00"
  },
  "error": null
}
```

### Response Body (Failure - HTTP 409 Conflict)

```json
{
  "error": "User already has a Lightning wallet"
}
```

---

## 3. AUTH METHOD

**Summary:** Mixed authentication strategy

### Public Endpoints
Public API endpoints do **NOT** require client authentication:
- `/wallet/create`
- `/wallet/:user_id`
- `/recovery/*`
- `/webhook/*`
- `/rates`
- `/ussd`

### Admin Endpoints
Admin endpoints (`/admin/*`) use **JWT tokens**.

**Flow:**
1. POST `/admin/login` with credentials:
   ```json
   {
     "username": "admin",
     "password": "yourpassword"
   }
   ```

2. Response (HTTP 200):
   ```json
   {
     "success": true,
     "message": "Login successful",
     "token": "eyJ...<jwt>"
   }
   ```

3. For subsequent admin requests, include:
   ```
   Authorization: Bearer eyJ...<jwt>
   ```

### Backend-Only Secrets
The following are **server-side only** (NOT supplied by frontend):
- `BREEZ_API_KEY` — Breez SDK authentication
- `PAYSTACK_SECRET_KEY` — Paystack webhook verification
- `SABI_NOSTR_NSEC` — Backend Nostr private key

---

## 4. OTHER ENDPOINTS (if ready)

### Wallet
- **GET** `/wallet/:user_id` — Retrieve wallet info by user UUID
  - Response: Same `WalletResponse` format (success + data)

### Recovery
- **POST** `/recovery/request` — Initiate recovery share requests via Nostr
  ```json
  {
    "wallet_id": "11111111-1111-1111-1111-111111111111",
    "helper_npubs": ["npub1helper1...", "npub1helper2..."]
  }
  ```
  Response:
  ```json
  {
    "success": true,
    "message": "Recovery request initiated successfully"
  }
  ```

- **POST** `/recovery/submit` — Submit encrypted recovery share from helper
  ```json
  {
    "wallet_id": "11111111-1111-1111-1111-111111111111",
    "encrypted_share": "<base64-or-hex-string>",
    "helper_pubkey": "npub1helper..."
  }
  ```
  Response:
  ```json
  {
    "success": true,
    "message": "Recovery share submitted successfully"
  }
  ```

### Webhooks
- **POST** `/webhook/breez` — Breez payment notifications
  - Header: `X-Breez-Signature` (signature verification in progress)
  - Payload: Breez webhook JSON format

- **POST** `/webhook/paystack` — Paystack payment confirmations
  - Header: `x-paystack-signature` (signature verification in progress)
  - Payload: Paystack webhook JSON format

### USSD
- **POST** `/ussd` — Africa's Talking USSD callback
  - Content-Type: `application/x-www-form-urlencoded`
  - Form fields (PascalCase): `SessionId`, `ServiceCode`, `PhoneNumber`, `Text`

### Public Rates
- **GET** `/rates` — Get cached Naira → BTC exchange rate
  ```json
  {
    "naira_to_btc": 0.00001234,
    "last_updated_at": "2025-11-30T12:34:56+00:00"
  }
  ```

### Admin
- **POST** `/admin/login` — Issue JWT token (details in section 3)

- **GET** `/admin/trades` — List all transactions (admin only)
  - Response: `{ "trades": [Transaction, ...] }`

- **POST** `/admin/manual-release` — Manually release funds for transaction
  ```json
  {
    "transaction_id": "11111111-1111-1111-1111-111111111111",
    "amount_sats": 50000,
    "recipient_nostr_pubkey": "npub1recipient...",
    "notes": "Optional admin notes"
  }
  ```
  Response:
  ```json
  {
    "success": true,
    "message": "Funds manually released for transaction ID: ...",
    "transaction_id": "11111111-1111-1111-1111-111111111111"
  }
  ```

### Health Check
- **GET** `/health/breez` — Breez node / SDK health check
  - Response: Health status (mocked in scaffold)

---

## 5. ERROR RESPONSE FORMAT

All errors follow a consistent JSON format:

```json
{
  "error": "Detailed error message here"
}
```

**HTTP Status Codes:**
- `400 Bad Request` — Invalid input (e.g., invalid UUID format, phone number validation)
- `409 Conflict` — Resource already exists (e.g., wallet already created for user)
- `401 Unauthorized` — Authentication failed (e.g., invalid credentials)
- `403 Forbidden` — Authenticated but not allowed (e.g., admin account inactive)
- `404 Not Found` — Resource not found (e.g., wallet does not exist)
- `422 Unprocessable Entity` — Validation error in request body
- `500 Internal Server Error` — Server error

**Example Error Responses:**

Invalid UUID:
```json
HTTP/1.1 400 Bad Request
{ "error": "Invalid user_id format" }
```

User already has wallet:
```json
HTTP/1.1 409 Conflict
{ "error": "User already has a Lightning wallet" }
```

Invalid credentials:
```json
HTTP/1.1 401 Unauthorized
{ "error": "Invalid username or password" }
```

---

## 6. BREEZ SDK ROLE

**Backend Responsibilities:**

The backend initializes and manages the Breez SDK server-side. See `bitcoin/breez.rs` and `services/breez_service.rs` in the codebase.

**Operations:**
1. **Wallet Creation** — Initialize Breez node using `BREEZ_MNEMONIC` and `BREEZ_API_KEY`
2. **Channel Management** — Open Lightning channels with initial liquidity
3. **Invoice Generation** — Create Lightning invoices for incoming payments
4. **Payments** — Send payments (on-chain or Lightning) for payouts and manual releases
5. **Webhooks** — Receive payment confirmations from Breez at `POST /webhook/breez`

**Frontend Interaction:**
- The Flutter frontend **does NOT** communicate directly with Breez SDK
- Call backend endpoints (e.g., `POST /wallet/create`)
- Backend initializes Breez SDK internally and returns wallet connection details
- No Breez mnemonic or API key is exposed to the frontend

---

## 7. SPECIAL HEADERS NEEDED

### Standard Headers
- **Content-Type:** `application/json` — for all JSON endpoints
- **Content-Type:** `application/x-www-form-urlencoded` — for USSD callbacks

### Authentication
- **Authorization:** `Bearer <JWT>` — for protected admin endpoints
  - Example: `Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...`

### Webhook Signature Verification (Server-side)
- **X-Breez-Signature** — Breez webhook signature (server verifies)
- **x-paystack-signature** — Paystack webhook signature (server verifies)

### Recommended (Optional)
- **X-Idempotency-Key** — Unique key for payment/webhook requests (prevents double-processing)
  - Example: `X-Idempotency-Key: 550e8400-e29b-41d4-a716-446655440000`

---

## Integration Checklist for Flutter Frontend

- [ ] Configure base URL (development or production)
- [ ] Implement user registration/login (to obtain `user_id`)
- [ ] Create wallet endpoint call (`POST /wallet/create`)
- [ ] Handle wallet response and store wallet connection details
- [ ] Fetch wallet info (`GET /wallet/:user_id`)
- [ ] Implement recovery request flow
- [ ] Implement error handling per status codes
- [ ] Add JWT token storage and refresh logic (if admin features needed)
- [ ] Test with actual backend instance

---

**Last Updated:** November 30, 2025
