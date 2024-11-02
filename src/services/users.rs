use actix_web::{
    cookie::{
        time::{Duration, OffsetDateTime},
        Cookie, SameSite,
    },
    get, post,
    web::{Data, Json},
    HttpMessage, HttpRequest, HttpResponse, Responder,
};
use diesel::{
    query_dsl::methods::{FilterDsl, FindDsl, SelectDsl},
    result::{DatabaseErrorKind, Error::DatabaseError},
    ExpressionMethods, OptionalExtension, RunQueryDsl, SelectableHelper,
};
use serde::Deserialize;
use validator::Validate;

use crate::{
    db::{
        connection::AppState,
        models::{CreateUser, User},
    },
    mail::{send::MailOptions, templates::verification::verification_template},
    utils::hashing::{decode_jwt, generate_jwt, verify_password, JwtMETHODS},
};

#[derive(Deserialize, Validate, Debug)]
pub struct RegisterRequest {
    #[validate(length(
        min = 6,
        max = 20,
        message = "Name length must be between 6 and 20 characters"
    ))]
    pub name: String,
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(
        min = 6,
        max = 20,
        message = "Password length must be between 6 and 20 characters"
    ))]
    pub password: String,
}

#[post("/users/register")]
async fn register(data: Data<AppState>, body: Json<RegisterRequest>) -> impl Responder {
    use crate::db::schema::users::dsl::users;
    let register_data = body.into_inner();

    // Validate the request body
    if let Err(validation_errors) = register_data.validate() {
        let register_error_messages: Vec<String> = validation_errors
            .field_errors()
            .into_iter()
            .map(|(field, errors)| {
                format!(
                    "Invalid {}: {}",
                    field,
                    errors
                        .into_iter()
                        .map(|e| e.message.clone().unwrap_or_else(|| "Invalid value".into()))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })
            .collect();

        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation error(s)",
            "details": register_error_messages,
        }));
    }

    let mut conn = match data.pool.get() {
        Ok(conn) => conn,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Database connection error"}));
        }
    };

    let existing_user = users
        .find(&register_data.email)
        .select(User::as_select())
        .first(&mut conn)
        .optional();

    match existing_user {
        Ok(Some(_)) => HttpResponse::Conflict().json(serde_json::json!({
            "error": "User already exists"
        })),
        Ok(None) => {
            let new_user = CreateUser::new(
                register_data.name,
                register_data.email,
                register_data.password,
            );

            match new_user {
                Ok(user) => {
                    match diesel::insert_into(users)
                        .values(user)
                        .returning(User::as_returning())
                        .get_result::<User>(&mut conn)
                    {
                        Ok(user) => {
                            let token = generate_jwt(user.id, JwtMETHODS::Default).unwrap();
                            let (
                                subject,
                                to,
                                html,
                                smtp_verification_name,
                                smtp_verification_user,
                                smtp_verification_email,
                            ) = verification_template(user.name, user.email, token);
                            let options = MailOptions {
                                user: smtp_verification_user,
                                user_name: smtp_verification_name,
                                user_email: smtp_verification_email,
                                to,
                                subject,
                                html_content: html,
                            };
                            MailOptions::send(options);
                            HttpResponse::Created().json(serde_json::json!({
                                "message": "User created successfully"
                            }))
                        }
                        Err(DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => {
                            HttpResponse::Conflict().json(serde_json::json!({
                                "error": "User already exists"
                            }))
                        }
                        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
                            "error": "An error occurred while creating the user"
                        })),
                    }
                }
                Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Invalid data",
                    "message": format!("An error occurred: {}", e)
                })),
            }
        }
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "An error occurred while registering the user"
        })),
    }
}

#[derive(Deserialize, Validate, Debug)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(
        min = 6,
        max = 20,
        message = "Password length must be between 6 and 20 characters"
    ))]
    pub password: String,
}

#[post("/users/login")]
async fn login(data: Data<AppState>, body: Json<LoginRequest>) -> impl Responder {
    use crate::db::schema::users::dsl::{email, users};
    let login_data = body.into_inner();

    // Validate the request body
    if let Err(validation_errors) = login_data.validate() {
        let login_error_messages: Vec<String> = validation_errors
            .field_errors()
            .into_iter()
            .map(|(field, errors)| {
                format!(
                    "Invalid {}: {}",
                    field,
                    errors
                        .into_iter()
                        .map(|e| e.message.clone().unwrap_or_else(|| "Invalid value".into()))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })
            .collect();

        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": login_error_messages
        }));
    }

    let mut conn = match data.pool.get() {
        Ok(conn) => conn,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Database connection error"}));
        }
    };

    let existing_user = users
        .filter(email.eq(&login_data.email))
        .first::<User>(&mut conn);

    match existing_user {
        Ok(user) => match verify_password(login_data.password, user.password.unwrap()) {
            Ok(_) => {
                let cookie_token = generate_jwt(user.id.to_string(), JwtMETHODS::Default).unwrap();
                let response_token = generate_jwt(user.id, JwtMETHODS::Login).unwrap();
                let cookie = Cookie::build("auth_token", cookie_token)
                    .http_only(true)
                    .secure(true)
                    .domain("rishabhportfolio.site")
                    .expires(OffsetDateTime::now_utc() + Duration::days(7))
                    .same_site(SameSite::Lax)
                    .path("/")
                    .finish();
                HttpResponse::Ok().cookie(cookie).json(serde_json::json!({
                    "token": response_token
                }))
            }
            Err(_) => HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid credentials"
            })),
        },
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "An error occurred while logging in"
        })),
    }
}

#[get("/users/check-auth")]
async fn check_auth(data: Data<AppState>, req: HttpRequest) -> impl Responder {
    use crate::db::schema::users::dsl::{id, users};

    let user_id_cookie = req.cookie("auth_token");

    if let Some(cookie) = user_id_cookie {
        let cookie_token = cookie.value();

        match decode_jwt(cookie_token) {
            Ok(cookie_data) => {
                let mut conn = match data.pool.get() {
                    Ok(conn) => conn,
                    Err(_) => {
                        return HttpResponse::InternalServerError()
                            .json(serde_json::json!({"error": "Database connection error"}));
                    }
                };
                let existing_user = users
                    .filter(id.eq(&cookie_data.sub))
                    .first::<User>(&mut conn);

                match existing_user {
                    Ok(user) => {
                        let response_token = generate_jwt(user.id, JwtMETHODS::Login).unwrap();
                        return HttpResponse::Ok().cookie(cookie).json(serde_json::json!({
                            "token": response_token
                        }));
                    }

                    Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": format!("No user found with this id {}", cookie_data.sub)
                    })),
                }
            }
            Err(err) => HttpResponse::Unauthorized().json(serde_json::json!({
                "error": format!("Invalid Cookie received with error:- {}", err)
            })),
        }
    } else {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "No Cookie received"
        }));
    }
}

#[get("/users/logout")]
async fn logout(data: Data<AppState>, req: HttpRequest) -> impl Responder {
    use crate::db::schema::users::dsl::{id, users};
    if let Some(authenticated_user_id) = req.extensions().get::<String>() {
        match data.pool.get() {
            Ok(mut conn) => {
                let user_id = authenticated_user_id.to_string();
                let user_exists = users.filter(id.eq(&user_id)).first::<User>(&mut conn);
                match user_exists {
                    Ok(_) => {
                        let cookie = Cookie::build("auth_token", "")
                            .http_only(true)
                            .secure(true)
                            .domain("rishabhportfolio.site") // Adjust domain as needed
                            .expires(OffsetDateTime::now_utc() - Duration::days(1)) // Set expired date
                            .same_site(SameSite::Lax)
                            .path("/")
                            .finish();
                        return HttpResponse::Ok().cookie(cookie).json(serde_json::json!({
                          "success": "User logged out successfully!"
                        }));
                    }
                    Err(_) => {
                        return HttpResponse::InternalServerError().json(serde_json::json!({
                            "error": format!("No user found with the provided id {}", user_id)
                        }))
                    }
                }
            }
            Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "An error occurred while connecting to the database"
            })),
        }
    } else {
        HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Missing or invalid token",

        }))
    }
}
