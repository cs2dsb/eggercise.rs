use axum::extract::FromRef;
use deadpool_sqlite::Pool;

use crate::AppState;

impl FromRef<AppState> for Pool {
    fn from_ref(state: &AppState) -> Self {
        // pool uses an Arc internally so clone is cheap
        state.pool.clone()
    }
}
