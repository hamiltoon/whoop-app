# Whoop API Integration — Design Spec

## Overview

A Rust web server (Axum) that authenticates with the Whoop API via OAuth 2.0 and exposes all available Whoop data through a local REST API. This serves as a foundation that can be extended with a frontend, additional integrations, or data persistence later.

## Tech Stack

- **Language:** Rust (2021 edition)
- **Web framework:** Axum
- **Async runtime:** Tokio
- **HTTP client:** Reqwest
- **Serialization:** Serde / serde_json
- **Environment config:** dotenvy
- **Session IDs:** uuid
- **Middleware:** tower-http (CORS, tracing)
- **Logging:** tracing / tracing-subscriber

## Project Structure

```
whoop-app/
├── Cargo.toml
├── .env.example          # WHOOP_CLIENT_ID, WHOOP_CLIENT_SECRET, REDIRECT_URI, PORT
├── src/
│   ├── main.rs           # Server startup, router setup
│   ├── config.rs         # Environment variables, configuration struct
│   ├── router.rs         # All routes assembled
│   ├── state.rs          # AppState (shared resources, token storage)
│   ├── whoop/
│   │   ├── mod.rs
│   │   ├── client.rs     # WhoopClient — HTTP client wrapping Whoop API
│   │   ├── models.rs     # Whoop API data types (serde structs)
│   │   └── auth.rs       # OAuth 2.0 flow (auth URL generation, token exchange, refresh)
│   └── handlers/
│       ├── mod.rs
│       ├── auth.rs       # /auth/login, /auth/callback, /auth/logout
│       ├── profile.rs    # /api/profile, /api/body
│       ├── cycles.rs     # /api/cycles, /api/cycles/:id
│       ├── recovery.rs   # /api/cycles/:id/recovery
│       ├── sleep.rs      # /api/sleep, /api/sleep/:id, /api/cycles/:id/sleep
│       └── workouts.rs   # /api/workouts, /api/workouts/:id
```

## OAuth 2.0 Authentication

### Flow

1. User visits `GET /auth/login`
2. Server redirects to Whoop authorization URL with all scopes
3. User authenticates with Whoop
4. Whoop redirects to `GET /auth/callback?code=XXX`
5. Server exchanges authorization code for access_token + refresh_token
6. Tokens stored in-memory, session cookie set
7. User redirected to `/`

### Whoop OAuth Details

- **Authorization URL:** `https://api.prod.whoop.com/oauth/oauth2/auth`
- **Token URL:** `https://api.prod.whoop.com/oauth/oauth2/token`
- **Scopes:** `read:recovery`, `read:cycles`, `read:workout`, `read:sleep`, `read:profile`, `read:body_measurement`
- **Callback URL:** `http://localhost:3000/auth/callback` (configurable via REDIRECT_URI)

### Token Management

- Tokens stored in `AppState`: `Arc<RwLock<HashMap<SessionId, TokenPair>>>`
- `TokenPair` contains `access_token`, `refresh_token`, and `expires_at`
- Automatic refresh on 401 response from Whoop API
- Session identified by a UUID cookie (`whoop_session`)

## API Endpoints

| Local Route | Method | Whoop Endpoint | Description |
|---|---|---|---|
| `/auth/login` | GET | — | Initiates OAuth flow |
| `/auth/callback` | GET | Token URL | Handles OAuth callback |
| `/auth/logout` | DELETE | Revoke endpoint | Logs out, revokes token |
| `/api/profile` | GET | `/v2/user/profile/basic` | User name, email |
| `/api/body` | GET | `/v2/user/measurement/body` | Height, weight, max HR |
| `/api/cycles` | GET | `/v2/cycle` | Paginated cycles |
| `/api/cycles/:id` | GET | `/v2/cycle/{id}` | Single cycle |
| `/api/cycles/:id/recovery` | GET | `/v2/cycle/{id}/recovery` | Recovery for cycle |
| `/api/cycles/:id/sleep` | GET | `/v2/cycle/{id}/sleep` | Sleep for cycle |
| `/api/sleep` | GET | `/v2/activity/sleep` | Paginated sleep records |
| `/api/sleep/:id` | GET | `/v2/activity/sleep/{id}` | Single sleep record |
| `/api/workouts` | GET | `/v2/activity/workout` | Paginated workouts |
| `/api/workouts/:id` | GET | `/v2/activity/workout/{id}` | Single workout |

### Pagination

Collection endpoints accept query parameters forwarded to Whoop API:
- `start` — ISO 8601 start time filter
- `end` — ISO 8601 end time filter
- `limit` — max items per page
- `next_token` — pagination cursor from previous response

### Response Format

All endpoints return Whoop API responses as-is (JSON passthrough), preserving the original structure. This keeps the API simple and avoids mapping drift.

## Error Handling

A single `AppError` enum implementing `IntoResponse`:

- `Unauthorized` → 401 (no session or expired token)
- `WhoopApiError { status, body }` → forwards Whoop's status code
- `TokenRefreshFailed` → 401 (refresh token expired, user must re-auth)
- `ConfigError` → 500 (missing env vars at startup)
- `InternalError` → 500 (unexpected errors)

All errors return JSON: `{ "error": "description" }`.

## Configuration

Environment variables (loaded from `.env`):

| Variable | Required | Default | Description |
|---|---|---|---|
| `WHOOP_CLIENT_ID` | Yes | — | OAuth client ID |
| `WHOOP_CLIENT_SECRET` | Yes | — | OAuth client secret |
| `REDIRECT_URI` | No | `http://localhost:3000/auth/callback` | OAuth callback URL |
| `PORT` | No | `3000` | Server port |
| `RUST_LOG` | No | `info` | Log level |

## Key Design Decisions

1. **JSON passthrough** — Whoop responses forwarded as-is. No intermediate mapping that could drift out of sync. Models are used for deserialization where needed (token handling, pagination) but collection data passes through.

2. **In-memory token storage** — Simple HashMap behind RwLock. Tokens are lost on restart (user must re-auth). Acceptable for a local development tool; can be swapped for SQLite/Redis later.

3. **Single-user focus** — While the session system supports multiple users, the primary use case is a single user running locally. No user management, no registration.

4. **No frontend initially** — Pure API server. Test with curl, Postman, or a browser for the OAuth flow. Frontend can be added later.

## Future Extensions (Out of Scope)

- Persistent token storage (SQLite)
- Frontend dashboard
- Data caching / historical storage
- Webhook support
- Export to CSV/JSON files
- Integration with other services
