use actix_web::ResponseError;
use derive_more::Display;

#[derive(Debug, Display)]
pub enum AuthorizationError {
    #[display(fmt = "Access denied, invalid token")]
    InvalidToken,
    #[display(fmt = "Access denied, no token")]
    NoToken,
}

impl ResponseError for AuthorizationError {}
