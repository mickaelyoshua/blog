#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Env {
    Development,
    Production,
}

impl Env {
    pub fn from_env() -> Self {
        match std::env::var("APP_ENV").as_deref() {
            Ok("production") => Self::Production,
            _ => Self::Development,
        }
    }
}
