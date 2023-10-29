use std::time::{SystemTime, UNIX_EPOCH};

use jsonwebtoken::{
    decode, encode, errors::ErrorKind, DecodingKey, EncodingKey, Header, Validation,
};
use serde::{Deserialize, Serialize};

const JWT_EXP_TIME: u128 = 3600000;

#[derive(Deserialize, Serialize, std::fmt::Debug)]
pub struct AccessTokenClaims {
    pub aud: String,
    pub exp: u128,
    pub iat: u128,
    pub iss: String,
    pub sub: String,

    pub email: String,
    pub name: String,
    pub picture: String,
}

pub fn create_access_token(
    jwt_secret: &str,
    jwt_iss: &str,
    jwt_aud: &str,
    uid: i32,
    email: &str,
    name: &str,
    picture: &str,
) -> Option<String> {
    let system_time = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_millis(),
        Err(e) => {
            eprintln!("error: {e}");
            return None;
        }
    };

    let token_claims = AccessTokenClaims {
        aud: jwt_aud.to_string(),
        iss: jwt_iss.to_string(),

        email: email.to_string(),
        name: name.to_string(),
        picture: picture.to_string(),
        sub: uid.to_string(),

        iat: system_time,
        exp: system_time + JWT_EXP_TIME,
    };

    match encode(
        &Header::default(),
        &token_claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    ) {
        Ok(token) => Some(token),
        Err(e) => {
            eprintln!("error: {e}");
            None
        }
    }
}

pub fn decode_access_token(
    access_token: &str,
    jwt_secret: &str,
    jwt_iss: &str,
    jwt_aud: &str,
) -> Option<AccessTokenClaims> {
    let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.set_audience(&[jwt_aud]);
    validation.set_issuer(&[jwt_iss]);

    match decode::<AccessTokenClaims>(
        &access_token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &validation,
    ) {
        Ok(td) => Some(td.claims),
        Err(e) => {
            match e.kind() {
                ErrorKind::ExpiredSignature => eprintln!("expired token"),
                ErrorKind::InvalidToken => eprintln!("invalid access token"),
                ErrorKind::InvalidAudience => eprintln!("invalid audience"),
                ErrorKind::InvalidIssuer => eprintln!("invalid issuer"),
                e => eprintln!("some other jwt error {:?}", e),
            }
            None
        }
    }
}
