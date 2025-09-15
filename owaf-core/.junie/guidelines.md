Project: owaf (Rust, Salvo web framework)

This document captures project-specific knowledge to speed up future development. It assumes you are an experienced Rust developer.

Build and configuration
- Rust toolchain: rustc >= 1.80 (Salvo requires this). If build fails with older toolchains, run: rustup update stable
- OS prerequisites (Windows): OpenSSL static MD via vcpkg is required by transitive dependencies. Install once:
  - vcpkg install openssl:x64-windows-static-md
- Running
  - cargo run
  - The server address is configured via config.toml (listen_addr). TLS can be enabled by setting [tls] with cert/key paths. When TLS is enabled, server binds with Rustls (see src/main.rs around tls handling).
- Logging
  - Logging level and formatting controlled by config (see src/config). Log file output is configured via tracing-appender guard initialized in main.
- Database
  - Default template assumes sqlite via SQLx. The pre-initialized DB lives under data/. If switching DB, update both config/config.toml and the ORM-specific .env used by sqlx tooling (entity generation, migrations, offline mode).

Testing
- We rely on tokio + salvo::test for integration-like tests without binding to a real port.
- Existing test: src/main.rs contains #[cfg(test)] module with tokio::test test_hello_world. It initializes config, builds the router via routers::root(), and uses TestClient against the in-process Service.
- Run all tests: cargo test
- Running a single test: cargo test test_hello_world
- Notes
  - Tests call config::init(); ensure config.toml exists and is consistent for test env. If tests should not hit a real DB, provide a test-specific configuration (e.g., sqlite file under data/ or an in-memory sqlite path) and ensure db::init() is not eagerly called for these request paths. The current test only exercises the root route and does not need DB if the route avoids DB access.
  - When adding routes that hit the DB, prefer constructing the Service with a state containing a test pool (e.g., SQLx SqlitePool::connect("sqlite::memory:")). Inject via Salvo Depot or router layer so tests can provide a separate pool.

How to add a new test
- For quick endpoint tests, embed under the module where the handlers live, or create a dedicated tests/ crate-style directory.
- Minimal example (works in this project):
  - Place inside any src file under a #[cfg(test)] module or under tests/hello.rs.
  - Example integration-style test using the existing router:
    
    // File: tests/smoke.rs
    use salvo::prelude::*;
    use salvo::test::{ResponseExt, TestClient};
    
    #[tokio::test]
    async fn smoke_root_ok() {
        owaf::config::init();
        let service = Service::new(owaf::routers::root());
        let body = TestClient::get("http://127.0.0.1:0")
            .send(&service)
            .await
            .take_string()
            .await
            .unwrap();
        assert_eq!(body, "Hello World from salvo");
    }
    
  - Notes:
    - We pass an arbitrary URL; TestClient routes internally to the provided Service without a real bind.
    - If your handler returns structured JSON, prefer asserting on deserialized payloads using serde.

Creating and running a simple test (verified)
- A smoke test similar to the above was validated locally by running cargo test; the existing test in src/main.rs passes and demonstrates the process. Use it as a reference when authoring additional tests.

Development conventions and tips
- Routing and error handling
  - The root router is composed in src/routers. Global hoops include CORS hoop and a 404 catcher (see hoops::cors_hoop and hoops::error_404). When adding routes, ensure they are attached under routers::root() so they inherit global hoops and error catcher.
- Error type
  - Unified error: AppError (src/error.rs) with AppResult<T> and JsonResult<T> type aliases in main.rs. Use json_ok(data) and empty_ok() helpers to return standardized JSON responses.
- OpenAPI and UI
  - When server starts, it prints URLs for Scalar UI (/scalar) and Login (/login). Behind TLS, URLs are https and bind address 0.0.0.0 is rewritten to 127.0.0.1 for convenience prints. Keep these prints in sync when changing routes.
- Configuration bootstrap
  - Always call config::init() before using config::get(). Tests and binaries do this explicitly. If you add new code paths that need configuration early, keep this ordering to avoid panics.
- Logging shutdown
  - Server registers shutdown_signal using tokio::signal (Ctrl+C and, on Unix, SIGTERM). If you add background tasks, store ServerHandle or JoinHandles and ensure graceful shutdown in 60s deadline is respected.
- Templates
  - Rinja is configured via rinja.toml. If adding views, keep template file names and paths consistent with views/ and Assets embedding.
- Assets
  - rust-embed is used for static assets under assets/. If you add or move assets, ensure the embed pattern matches and MIME types are handled as expected in static routes.

Maintenance checklist
- After changing DB config or enabling TLS, test both run (cargo run) and tests (cargo test) locally.
- For Windows, if OpenSSL linkage errors occur, re-run vcpkg install and ensure VCPKG_ROOT is available in the environment for the VS/toolchain session.
- Keep Rust version in Cargo.toml in sync with the minimum supported toolchain mentioned here.
