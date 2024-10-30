use std::{
    future::{ready, Future, Ready},
    pin::Pin,
};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorUnauthorized,
    Error, HttpMessage,
};
use diesel::QueryDsl;

use crate::{db::schema::users::dsl::users, utils::hashing::decode_jwt};

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
        let a = req.headers().get("Authorization");
        println!("{:?}", a);

        let path = req.path();
        let posts_pth = path.starts_with("/posts");
        let users_pth = path.starts_with("/users");

        if users_pth {
            let protected_routes = vec!["/logout"];
            let routes = protected_routes.iter();
            for route in routes {
                if path.ends_with(route) {
                    if let Some(cookie) = user_id_cookie {
                        let cookie_token = cookie.value();

                        match decode_jwt(cookie_token) {
                            Ok(cookie_data) => {
                                let authorization = req.headers().get("Authorization");

                                if let Some(auth_data) = authorization {
                                    let data = auth_data.to_str();
                                    match data {
                                        Ok(auth_val) => {
                                            let header_token = &auth_val[7..];

                                            match decode_jwt(header_token) {
                                                Ok(header_data) => {
                                                    if cookie_data.sub == header_data.sub {
                                                        // let existing_user =
                                                        //     users.find(&header_data.sub);

                                                        req.extensions_mut()
                                                            .insert(header_data.sub);

                                                        let fut = self.service.call(req);
                                                        return Box::pin(async move {
                                                            let res = fut.await?;
                                                            Ok(res)
                                                        });
                                                    } else {
                                                        return Box::pin(async move {
                                                            Err(ErrorUnauthorized(
                                                                "User ID mismatch between cookie and header!",
                                                            ))
                                                        });
                                                    }
                                                }
                                                Err(err) => {
                                                    return Box::pin(async move {
                                                        Err(ErrorUnauthorized(format!(
                                                            "Invalid JWT in Authorization header: {}",
                                                            err
                                                        )))
                                                    });
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            return Box::pin(async move {
                                                Err(ErrorUnauthorized(format!(
                                                    "Error converting Authorization header to string: {}",
                                                    e
                                                )))
                                            });
                                        }
                                    }
                                } else {
                                    return Box::pin(async move {
                                        Err(ErrorUnauthorized("Authorization header not found!"))
                                    });
                                }
                            }
                            Err(err) => {
                                return Box::pin(async move {
                                    Err(ErrorUnauthorized(format!(
                                        "Invalid JWT in auth_token cookie: {}",
                                        err
                                    )))
                                });
                            }
                        }
                    } else {
                        return Box::pin(async move {
                            Err(ErrorUnauthorized("auth_token cookie not found!"))
                        });
                    }
                } else {
                    let fut = self.service.call(req);
                    return Box::pin(async move {
                        let res = fut.await?;
                        Ok(res)
                    });
                }
            }
        }

        if posts_pth {
            let protected_routes = vec!["/update", "/create", "/delete"];
            let routes = protected_routes.iter();
            for route in routes {
                if path.ends_with(route) {
                    if let Some(cookie) = user_id_cookie {
                        let cookie_token = cookie.value();

                        match decode_jwt(cookie_token) {
                            Ok(cookie_data) => {
                                let authorization = req.headers().get("Authorization");

                                if let Some(auth_data) = authorization {
                                    let data = auth_data.to_str();
                                    match data {
                                        Ok(auth_val) => {
                                            let header_token = &auth_val[7..];

                                            match decode_jwt(header_token) {
                                                Ok(header_data) => {
                                                    if cookie_data.sub == header_data.sub {
                                                        req.extensions_mut()
                                                            .insert(header_data.sub);

                                                        let fut = self.service.call(req);
                                                        return Box::pin(async move {
                                                            let res = fut.await?;
                                                            Ok(res)
                                                        });
                                                    } else {
                                                        return Box::pin(async move {
                                                            Err(ErrorUnauthorized(
                                                                "User ID mismatch between cookie and header!",
                                                            ))
                                                        });
                                                    }
                                                }
                                                Err(err) => {
                                                    return Box::pin(async move {
                                                        Err(ErrorUnauthorized(format!(
                                                            "Invalid JWT in Authorization header: {}",
                                                            err
                                                        )))
                                                    });
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            return Box::pin(async move {
                                                Err(ErrorUnauthorized(format!(
                                                    "Error converting Authorization header to string: {}",
                                                    e
                                                )))
                                            });
                                        }
                                    }
                                } else {
                                    return Box::pin(async move {
                                        Err(ErrorUnauthorized("Authorization header not found!"))
                                    });
                                }
                            }
                            Err(err) => {
                                return Box::pin(async move {
                                    Err(ErrorUnauthorized(format!(
                                        "Invalid JWT in auth_token cookie: {}",
                                        err
                                    )))
                                });
                            }
                        }
                    } else {
                        return Box::pin(async move {
                            Err(ErrorUnauthorized("auth_token cookie not found!"))
                        });
                    }
                } else {
                    let fut = self.service.call(req);
                    return Box::pin(async move {
                        let res = fut.await?;
                        Ok(res)
                    });
                }
            }
            let fut = self.service.call(req);
            return Box::pin(async move {
                let res = fut.await?;
                Ok(res)
            });
        } else {
            let fut = self.service.call(req);
            return Box::pin(async move {
                let res = fut.await?;
                Ok(res)
            });
        }
    }
}
