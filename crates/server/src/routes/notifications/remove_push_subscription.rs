use axum::Json;
use shared::api::error::{Nothing, ServerError};
use tracing::{debug, warn};

use crate::{db::DatabaseConnection, UserState};

pub async fn remove_push_subscription(
    DatabaseConnection(conn): DatabaseConnection,
    user_state: UserState,
) -> Result<Json<()>, ServerError<Nothing>> {
    conn.interact(move |conn| {
        let mut user = user_state.id.fetch_full_user(conn)?;
        if user.push_notification_subscription.is_some() {
            debug!("Removing push subscription for user_id {}", user.id);
            user.push_notification_subscription = None;
            user.update(conn)?;
        } else {
            warn!(
                "Attempt to remove push subscription for user_id {} but it wasn't configured",
                user.id
            );
        }

        Ok::<_, ServerError<_>>(())
    })
    .await??;

    Ok(Json(()))
}
