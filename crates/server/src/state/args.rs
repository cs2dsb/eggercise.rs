use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
};

use crate::cli::Cli;

#[derive(Debug)]
pub struct Args(Arc<Cli>);

impl From<Arc<Cli>> for Args {
    fn from(args: Arc<Cli>) -> Self {
        Args(args)
    }
}

impl Deref for Args {
    type Target = Arc<Cli>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Args {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Args
where
    S: Send + Sync,
    Arc<Cli>: FromRef<S>,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let args = <Arc<Cli>>::from_ref(state);
        Ok(args.into())
    }
}
