use crate::utils::hashing::decode_jwt;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorUnauthorized,
    Error, HttpMessage,
};
use std::{
    future::{ready, Future, Ready},
    pin::Pin,
};

pub struct Authentication;

impl<S, B> Transform<S, ServiceRequest> for Authentication
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthenticationMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthenticationMiddleware { service }))
    }
}

pub struct AuthenticationMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthenticationMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + 'static>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let user_id_cookie = req.cookie("auth_token");

        let path = req.path();
        let protected_routes = match path {
            p if p.starts_with("/posts") => vec!["/update", "/create", "/delete"],
            p if p.starts_with("/users") => vec!["/logout"],
            _ => return Box::pin(self.service.call(req)),
        };

        for route in protected_routes {
            if path.ends_with(route) {
                if let Some(cookie) = user_id_cookie {
                    let cookie_token = cookie.value();
                    return match validate_jwt(&req, cookie_token) {
                        Ok(sub) => {
                            req.extensions_mut().insert(sub);
                            Box::pin(self.service.call(req))
                        }
                        Err(e) => Box::pin(async move { Err(ErrorUnauthorized(e)) }),
                    };
                } else {
                    return Box::pin(async move {
                        Err(ErrorUnauthorized("auth_token cookie not found!"))
                    });
                }
            }
        }

        Box::pin(self.service.call(req))
    }
}

fn validate_jwt(req: &ServiceRequest, cookie_token: &str) -> Result<String, String> {
    match decode_jwt(cookie_token) {
        Ok(cookie_data) => {
            let authorization = req.headers().get("Authorization");

            if let Some(auth_data) = authorization {
                let auth_val = auth_data
                    .to_str()
                    .map_err(|_| "Error converting Authorization header to string")?;
                let header_token = &auth_val[7..];

                match decode_jwt(header_token) {
                    Ok(header_data) => {
                        if cookie_data.sub == header_data.sub {
                            Ok(header_data.sub)
                        } else {
                            Err("User ID mismatch between cookie and header!".into())
                        }
                    }
                    Err(err) => Err(format!("Invalid JWT in Authorization header: {}", err)),
                }
            } else {
                Err("Authorization header not found!".into())
            }
        }
        Err(err) => Err(format!("Invalid JWT in auth_token cookie: {}", err)),
    }
}
