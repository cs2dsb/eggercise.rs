use axum::Json;
use shared::api::{
    error::{Nothing, ServerError},
    payloads::{UpdateSubscriptionRequest, UpdateSubscriptionResponse},
};
use tracing::warn;

use crate::{db::DatabaseConnection, UserState};

pub async fn update_push_subscription(
    DatabaseConnection(conn): DatabaseConnection,
    user_state: UserState,
    Json(req): Json<UpdateSubscriptionRequest>,
) -> Result<Json<UpdateSubscriptionResponse>, ServerError<Nothing>> {
    conn.interact(move |conn| {
        let mut user = user_state.id.fetch_full_user(conn)?;
        if user.push_notification_subscription.is_some() {
            warn!("Overwriting push subscription for user_id {}", user.id);
        }
        user.push_notification_subscription = Some(req.subscription);
        user.update(conn)?;

        Ok::<_, ServerError<_>>(())
    })
    .await??;

    let update_subscription_response = UpdateSubscriptionResponse {};
    Ok(Json(update_subscription_response))
}
