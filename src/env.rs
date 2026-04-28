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

#[cfg(test)]
mod tests {
    use super::*;

    // All cases run sequentially in a single #[test] because Rust 2024 makes
    // env mutation unsafe and the global env is shared across the test binary.
    // Splitting into multiple #[test] fns would let cargo's parallel runner
    // race them.
    #[test]
    fn from_env_matches_only_literal_production() {
        unsafe {
            // Unset → Development (fail-safe default).
            std::env::remove_var("APP_ENV");
            assert_eq!(Env::from_env(), Env::Development);

            // Empty string → Development.
            std::env::set_var("APP_ENV", "");
            assert_eq!(Env::from_env(), Env::Development);

            // Wrong case → Development. The match is case-sensitive on purpose
            // so a deploy with `APP_ENV=Production` does NOT enable HSTS.
            std::env::set_var("APP_ENV", "Production");
            assert_eq!(Env::from_env(), Env::Development);

            // Common typo → Development.
            std::env::set_var("APP_ENV", "prod");
            assert_eq!(Env::from_env(), Env::Development);

            // Stray whitespace → Development. We do NOT trim.
            std::env::set_var("APP_ENV", " production ");
            assert_eq!(Env::from_env(), Env::Development);

            // Exact match → Production.
            std::env::set_var("APP_ENV", "production");
            assert_eq!(Env::from_env(), Env::Production);

            // Cleanup so we don't pollute later tests in the same binary.
            std::env::remove_var("APP_ENV");
        }
    }
}
