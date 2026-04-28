#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Env {
    Development,
    Production,
}

impl Env {
    // Anything other than the literal string "production" — including unset,
    // empty, or typos like "prod" — maps to Development. This is fail-safe for
    // local dev but means a misconfigured prod deploy will silently disable
    // production-only behaviors (e.g. HSTS in security_headers).
    pub fn from_env() -> Self {
        match std::env::var("APP_ENV").as_deref() {
            Ok("production") => Self::Production,
            _ => Self::Development,
        }
    }
}
