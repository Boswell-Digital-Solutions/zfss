## 3. Tech Stack

### Runtime

| Component | Version | Purpose |
|-----------|---------|---------|
| Rust | 2024 edition | Backend language |
| TypeScript | 5.x | Frontend language |
| Node.js | 18+ | Build tooling |
| PostgreSQL | 14+ | ZFSS operational feedback store |

### Framework

| Component | Version | Purpose |
|-----------|---------|---------|
| Tauri | 2.0 | Desktop app framework |
| Vite | 5.x | Frontend build system |

### Rust Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| tauri | 2.0 | Desktop framework core |
| tauri-plugin-global-shortcut | 2.0 | Ctrl+Alt+Z hotkey |
| sqlx | 0.7 | Async PostgreSQL driver |
| tokio | 1.x | Async runtime (multi-threaded) |
| serde / serde_json | 1.x | Serialization |
| uuid | 1.x | v4 UUID generation |
| chrono | 0.4 | Timezone-aware timestamps |
| directories | 5.x | OS-specific app data paths |
| anyhow | 1.x | Error handling |
| thiserror | 1.x | Typed errors |
| regex | 1.x | Input validation |
| rand | 0.8 | Random generation |
| dotenvy | 0.15 | .env file loading |

### Frontend Dependencies

| Package | Version | Purpose |
|---------|---------|---------|
| @tauri-apps/api | 2.0 | Tauri IPC bridge |

### Build Tooling

| Tool | Purpose |
|------|---------|
| @tauri-apps/cli | Desktop app build/dev |
| vite | Dev server + bundling |
| typescript | Type checking |

### Build Targets

- Linux: `.deb`, `.appimage`
- Dev server: `http://localhost:5173`
- Frontend output: `dist/`
- Browser targets: ES2021, Chrome 100+, Safari 13+
