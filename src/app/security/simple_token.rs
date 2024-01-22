use super::error::AuthorizationError;
use crate::app::db::AppState;
use crate::app::messages::TOKEN_COOKIE;
use actix_web::dev::{forward_ready, Service, ServiceResponse, Transform};
use actix_web::{dev::ServiceRequest, Error};
use std::{
    future::{ready, Future, Ready},
    pin::Pin,
    rc::Rc,
};

pub struct SimpleToken;

// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S, ServiceRequest> for SimpleToken
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = SimpleTokenMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SimpleTokenMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct SimpleTokenMiddleware<S> {
    /// The next service to call
    service: Rc<S>,
}

// This future doesn't have the requirement of being `Send`.
// See: futures_util::future::LocalBoxFuture
type LocalBoxFuture<T> = Pin<Box<dyn Future<Output = T> + 'static>>;

// `S`: type of the wrapped service
// `B`: type of the body - try to be generic over the body where possible
impl<S, B> Service<ServiceRequest> for SimpleTokenMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<Result<Self::Response, Self::Error>>;

    // This service is ready when its next service is ready
    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        log::debug!(
            "IP Filtering security control. You requested: {}",
            req.path()
        );

        let svc = self.service.clone();

        Box::pin(async move {
            let _peer_addr = match req.peer_addr() {
                Some(ip) => ip.ip().to_string(),
                None => "Unknown".to_string(),
            };

            let conn_info = req.connection_info().clone();
            let real_remote_addr = conn_info.realip_remote_addr().unwrap_or("unknown");

            let app_data = req.app_data::<AppState>().unwrap();

            let success = match app_data.config.security.auth_tokens.len() {
                // If no token set
                0 => true,
                _ => {
                    let token = match req.cookie(TOKEN_COOKIE) {
                        Some(cookie) => cookie.value().to_owned(),
                        None => {
                            return Err(Error::from(AuthorizationError::NoToken).into());
                        }
                    };
                    app_data.config.security.auth_tokens.contains(&token)
                }
            };

            match success {
                true => (),
                false => {
                    log::error!(
                        "Unauthorized access attempt for ip {} for {}",
                        real_remote_addr,
                        req.path()
                    );
                    return Err(Error::from(AuthorizationError::InvalidToken).into());
                }
            }
            log::info!(
                "Authorized access attempt for ip {} for {}",
                real_remote_addr,
                req.path()
            );

            let res = svc.call(req).await?;

            Ok(res)
        })
    }
}
