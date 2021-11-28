use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

use super::super::service;
use actix_web::body::MessageBody;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::http::HeaderValue;
use actix_web::{error, Error};
use futures::future::{ok, Ready};
use futures::Future;
use log::info;
use std::sync::Arc;

pub struct Auth(pub Arc<service::Service>);

impl<S, B> Transform<S> for Auth
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddleware {
            service: Rc::new(RefCell::new(service)),
            db_service: self.0.clone(),
        })
    }
}

pub struct AuthMiddleware<S> {
    service: Rc<RefCell<S>>,
    db_service: Arc<service::Service>,
}

impl<S, B> Service for AuthMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        let mut svr = self.service.clone();
        let db_service_clone = Arc::clone(&self.db_service);
        Box::pin(async move {
            let path = req.path().to_string();
            if path == "/api/login" || path == "/" || path.starts_with("/admin") {
                return Ok(svr.call(req).await?);
            }
            let value = HeaderValue::from_str("").unwrap();
            let token = req.headers().get("token").unwrap_or(&value);
            if token.len() > 0 {
                let token = token.to_str().unwrap().to_string();
                match db_service_clone.account_service.get_by_token(token.clone()) {
                    Err(e) => {
                        info!("{}", &e.to_string());
                        Err(error::ErrorUnauthorized("Unauthorized"))
                    }
                    Ok(account) => match account.clone() {
                        None => Err(error::ErrorUnauthorized("Unauthorized")),
                        Some(_) => Ok(svr.call(req).await?),
                    },
                }
            } else {
                Err(error::ErrorUnauthorized("Unauthorized"))
            }
        })
    }
}
