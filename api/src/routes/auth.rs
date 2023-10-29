// Routes under the /auth path

use actix_web::{get, post, web, HttpResponse};
use base64::{engine::general_purpose, Engine};
use entities::user::{self, Entity as User};
use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

use crate::{
    app_state::AppState,
    jwt::{self, AccessTokenClaims},
};

#[derive(Deserialize)]
pub struct GoogleLoginReqBody {
    gid_token: String,
}

#[derive(Serialize)]
pub struct GoogleLoginRespBody<'a> {
    access_token: Option<&'a str>,
    msg: Option<&'a str>,
}

impl GoogleLoginRespBody<'_> {
    pub fn access_token<'a>(token: &'a str) -> GoogleLoginRespBody<'a> {
        GoogleLoginRespBody {
            access_token: Some(token),
            msg: None,
        }
    }

    pub fn msg<'a>(msg: &'a str) -> GoogleLoginRespBody<'a> {
        GoogleLoginRespBody {
            access_token: None,
            msg: Some(msg),
        }
    }
}

#[post("/auth/google-login")]
pub async fn handle_google_login(
    body: web::Json<GoogleLoginReqBody>,
    state: web::Data<AppState>,
) -> HttpResponse {
    // First get the Google Id Token Payload
    let encoded_gid_payload = match get_gid_payload(&body.gid_token) {
        Some(payload) => payload,
        None => {
            return HttpResponse::Unauthorized().json(GoogleLoginRespBody::msg(
                "Unable to get Google Id Token Payload",
            ))
        }
    };
    // Now, decode the payload
    let gid_payload = match decode_gid_payload(&encoded_gid_payload) {
        Some(payload) => payload,
        None => {
            return HttpResponse::Unauthorized().json(GoogleLoginRespBody::msg(
                "Unable to deserialize Google Id Token Payload",
            ))
        }
    };

    // Once the payload has been parsed, we need to check for sufficient Google OAuth scopes.
    // If the email is present, then the rest of them will also be.
    if let None = gid_payload.email {
        return HttpResponse::Unauthorized().json(GoogleLoginRespBody::msg(
            "Insufficient Google OAuth scopes.",
        ));
    }

    let user_email = gid_payload.email.unwrap();

    // Now, we must check if the user with the given email has already been registered.
    let maybe_registed_user: Option<user::Model> = match User::find()
        .filter(user::Column::Email.contains(&user_email))
        .one(&state.db)
        .await
    {
        Ok(data) => data,
        Err(e) => {
            eprintln!("error: {e}");
            return HttpResponse::InternalServerError()
                .json(GoogleLoginRespBody::msg("Unable to make database query"));
        }
    };

    if let Some(user) = maybe_registed_user {
        match jwt::create_access_token(
            &state.jwt_sec,
            &state.jwt_iss,
            &state.jwt_aud,
            user.id,
            &user_email,
            &user.name,
            &user.picture,
        ) {
            Some(token) => HttpResponse::Ok().json(GoogleLoginRespBody::access_token(&token)),
            None => HttpResponse::InternalServerError()
                .json(GoogleLoginRespBody::msg("Unable to generate access token")),
        }
    } else {
        // If the user does not exist, we must make an entity.
        let new_user_model = user::ActiveModel {
            email: ActiveValue::Set(user_email.clone()),
            picture: ActiveValue::Set(gid_payload.picture.unwrap()),
            name: ActiveValue::Set(gid_payload.name.unwrap()),
            ..Default::default()
        };

        // Now, we can save the user.
        let user: user::Model = match new_user_model.insert(&state.db).await {
            Ok(model) => model,
            Err(e) => {
                eprintln!("error: {e}");
                return HttpResponse::InternalServerError().json(GoogleLoginRespBody::msg(
                    "Unable to make database insertion",
                ));
            }
        };

        match jwt::create_access_token(
            &state.jwt_sec,
            &state.jwt_iss,
            &state.jwt_aud,
            user.id,
            &user_email,
            &user.name,
            &user.picture,
        ) {
            Some(token) => HttpResponse::Ok().json(GoogleLoginRespBody::access_token(&token)),
            None => HttpResponse::InternalServerError()
                .json(GoogleLoginRespBody::msg("Unable to generate access token")),
        }
    }
}

fn get_gid_payload<'a>(gid_token: &'a str) -> Option<&'a str> {
    let frags: Vec<&str> = gid_token.split('.').collect();
    if frags.len() < 2 {
        None
    } else {
        Some(frags[1])
    }
}

#[derive(Deserialize, std::fmt::Debug)]
struct GoogleIdTokenPayload {
    iss: String,
    sub: String,
    azp: String,
    aud: String,
    iat: u128,
    exp: u128,

    email: Option<String>,
    email_verified: Option<bool>,
    name: Option<String>,
    picture: Option<String>,
    given_name: Option<String>,
    family_name: Option<String>,
    locale: Option<String>,
}

fn decode_gid_payload(gid_payload: &str) -> Option<GoogleIdTokenPayload> {
    let decoded_bytes = match general_purpose::STANDARD_NO_PAD.decode(gid_payload) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("error decoding gid payload: {e}");
            return None;
        }
    };

    match serde_json::from_slice::<GoogleIdTokenPayload>(&decoded_bytes) {
        Ok(val) => Some(val),
        Err(e) => {
            eprintln!("unable to deserialize payload: {e}");
            None
        }
    }
}

#[derive(Serialize)]
pub struct VerifyTokenResp {
    claims: Option<AccessTokenClaims>,
}

#[derive(Deserialize)]
pub struct VerifyTokenReqQuery {
    token: Option<String>,
}

#[get("/auth/verify-access-token")]
pub async fn handle_verify_access_token(
    state: web::Data<AppState>,
    query: web::Query<VerifyTokenReqQuery>,
) -> HttpResponse {
    // The "token" query param must be there.
    let access_token = match &query.token {
        Some(token) => token,
        None => {
            return HttpResponse::Unauthorized().json(VerifyTokenResp { claims: None });
        }
    };

    let decode_result =
        jwt::decode_access_token(access_token, &state.jwt_sec, &state.jwt_iss, &state.jwt_aud);

    HttpResponse::Ok().json(VerifyTokenResp {
        claims: decode_result,
    })
}
