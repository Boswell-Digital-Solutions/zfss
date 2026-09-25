## 5. Configuration & Environment

### Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `ZFSS_DATABASE_URL` | Yes | — | PostgreSQL connection string |
| `DATABASE_URL` | Fallback | — | Used if `ZFSS_DATABASE_URL` not set |
| `ZFSS_USER_ID` | No | auto-generated | Current user UUID |
| `ZFSS_USER_ROLE` | No | `steward` | User role (steward/operator/engineer/ai) |
| `ZFSS_ALWAYS_ON_TOP` | No | `false` | Keep capture window above all windows |

### Settings (config/settings.rs)

```rust
pub struct Settings {
    pub database_url: String,      // PostgreSQL connection URL
    pub user_id: String,           // Current user identifier
    pub user_role: UserRole,       // Role for authority checks
    pub always_on_top: bool,       // Window z-order preference
}
```

Settings are loaded from environment on startup. The database URL is validated to ensure it points to PostgreSQL (must start with `postgresql://` or `postgres://`).

### Device ID

A persistent device UUID is stored at:
```
~/.local/share/zfss/device_id.txt
```

Generated on first launch via `uuid::Uuid::new_v4()`. Used to identify the device across sessions.

### .env File

```bash
ZFSS_DATABASE_URL=postgresql://localhost/zfss
```

Loaded via `dotenvy` on application startup.

### Database Connection

The `db/pool.rs` module creates a `PgPool` with:
- Connection string from Settings
- Health check on startup (`SELECT 1`)
- Pool shared via `AppState` (Arc-wrapped)

### Tauri Configuration (tauri.conf.json)

| Setting | Value |
|---------|-------|
| Product name | `zfss` |
| Identifier | `com.forge.zfss` |
| Window title | `ZFSS - Signal Capture` |
| Window size | 600 × 400 |
| Dev URL | `http://localhost:5173` |
| Build output | `../dist` |
| Bundle targets | `deb`, `appimage` |
