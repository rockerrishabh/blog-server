use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum JwtMETHODS {
    Default,
    PasswordReset,
    Login,
}

pub fn hash_password(password: String) -> Result<String, argon2::password_hash::Error> {
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hashed_password| hashed_password.to_string())
}

pub fn verify_password(
    password: String,
    hashed_password: String,
) -> Result<(), argon2::password_hash::Error> {
    let argon2 = Argon2::default();
    let parsed_hash = PasswordHash::new(&hashed_password)?;

    argon2.verify_password(password.as_bytes(), &parsed_hash)
}

pub fn generate_jwt(
    user_id: String,
    method: JwtMETHODS,
) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration: u64;
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    match method {
        JwtMETHODS::Default => {
            expiration = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs()
                + 60 * 60 * 24 * 7;
        }
        JwtMETHODS::PasswordReset => {
            expiration = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs()
                + 60 * 60 * 24;
        }
        JwtMETHODS::Login => {
            expiration = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs()
                + 60 * 60;
        }
    }

    let claims = Claims {
        sub: user_id,
        exp: expiration as usize,
    };

    let token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )?;
    Ok(token)
}

pub fn decode_jwt(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let mut validation = Validation::new(Algorithm::HS256);
    validation.leeway = 0;
    validation.validate_exp = true;
    validation.validate_nbf = false;

    let decoded_token = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &validation,
    )?;

    Ok(decoded_token.claims)
}
