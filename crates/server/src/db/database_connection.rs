use std::ops::{Deref, DerefMut};

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
};
use deadpool_sqlite::{Object, Pool};

use crate::internal_error;

#[derive(Debug)]
pub struct DatabaseConnection(pub Object);

impl From<Object> for DatabaseConnection {
    fn from(conn: Object) -> Self {
        DatabaseConnection(conn)
    }
}

impl Deref for DatabaseConnection {
    type Target = Object;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DatabaseConnection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for DatabaseConnection
where
    S: Send + Sync,
    Pool: FromRef<S>,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let pool = Pool::from_ref(state);

        // Get connection from the pool
        let conn = pool.get().await.map_err(internal_error)?;

        Ok(conn.into())
    }
}
