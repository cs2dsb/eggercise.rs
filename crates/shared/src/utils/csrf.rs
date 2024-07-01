use std::{error::Error, fmt::Display};

use gloo_net::http::{Request, RequestBuilder, Response};
use leptos::{provide_context, use_context, Owner};
use tracing::debug;

use crate::api::{error::FrontendError, Object, CSRF_HEADER};

#[derive(Debug, Clone)]
pub struct Csrf {
    token: String,
}

impl Csrf {
    pub async fn get<E>() -> Result<Self, FrontendError<E>>
    where
        E: Error + Display,
    {
        Ok(if let Some(csrf) = use_context::<Self>() {
            csrf
        } else {
            let response = Request::get(Object::Ping.path()).send().await?;

            let token = Self::token_from_response(&response)?;

            let csrf = Csrf {
                token,
            };
            csrf.provide_context();
            csrf
        })
    }

    pub fn add_to(&self, builder: RequestBuilder) -> RequestBuilder {
        builder.header(CSRF_HEADER, self.token())
    }

    pub fn update_from<E>(&mut self, response: &Response) -> Result<(), FrontendError<E>>
    where
        E: Error + Display,
    {
        let token = Self::token_from_response(response)?;
        self.update(token);
        Ok(())
    }

    pub fn provide_from<E>(response: &Response) -> Result<(), FrontendError<E>>
    where
        E: Error + Display,
    {
        let token = Self::token_from_response(response)?;

        Csrf {
            token,
        }
        .provide_context();

        Ok(())
    }

    fn token_from_response<E>(response: &Response) -> Result<String, FrontendError<E>>
    where
        E: Error + Display,
    {
        Ok(response
            .headers()
            .get(CSRF_HEADER)
            .ok_or(FrontendError::Client {
                message: "Csrf token header missing from response headers".to_string(),
            })?)
    }

    fn provide_context(&self) {
        let owner = Owner::current().expect("No owner!");
        debug!("Providing CSRF context with owner: {:?}", owner);
        provide_context(self.clone());
    }

    pub fn token(&self) -> &str {
        self.token.as_ref()
    }

    pub fn update(&mut self, token: String) {
        self.token = token;
        self.provide_context();
    }
}
