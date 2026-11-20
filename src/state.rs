use std::sync::Arc;

use sqlx::PgPool;

use crate::config::Config;
use crate::oidc::keys::SigningKeys;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub cfg: Arc<Config>,
    pub keys: Arc<SigningKeys>,
}
