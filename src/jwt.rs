use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{Algorithm, decode, DecodingKey, encode, EncodingKey, Header, Validation};
use rocket::http::{Status, CookieJar, Cookie};
use serde::{Deserialize, Serialize};

use crate::role::Role;
use crate::user::User;
use crate::error::Problem;
use uuid::Uuid;

pub static USER_AUTH_KEY: &'static [u8] = include_bytes!("../jwt-keys/user_auth");
pub static USER_AUTH_PUB_KEY: &'static [u8] = include_bytes!("../jwt-keys/user_auth.pub");

pub static AUTH_COOKIE_NAME: &'static str = "jwt_auth";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRolesToken {
    #[serde(with = "jwt_numeric_date")]
    iat: DateTime<Utc>,
    #[serde(with = "jwt_numeric_date")]
    exp: DateTime<Utc>,
    user: Uuid,
    roles: Vec<Role>,
}

impl UserRolesToken {
    pub fn new(user: &User) -> UserRolesToken {
        let now = Utc::now();
        UserRolesToken {
            iat: now,
            exp: now + Duration::weeks(1),
            user: user.id.clone(),
            roles: user.user_roles.clone(),
        }
    }

    pub fn has_role(&self, role: &Role) -> bool {
        self.roles.contains(role)
    }

    pub fn encode_jwt(&self) -> Result<String, jsonwebtoken::errors::Error> {
        let header = Header::new(Algorithm::PS256);
        let key = EncodingKey::from_rsa_pem(USER_AUTH_PUB_KEY)?;

        Ok(encode(&header, &self, &key)?)
    }

    pub fn cookie<'c>(self) -> Result<Cookie<'c>, jsonwebtoken::errors::Error> {
        Ok(Cookie::build(
            AUTH_COOKIE_NAME,
            self.encode_jwt()?,
        )
            .secure(true)
            .path("/")
            .http_only(true)
            .finish()
        )
    }
}

fn auth_problem<S: Into<String>>(detail: S) -> Problem {
    Problem::new_untyped(
        Status::Unauthorized,
        "Unable to authorize user.",
    )
        .detail(detail)
        .clone()
}

pub fn extract_claims(cookies: CookieJar) -> Result<UserRolesToken, Problem> {
    let token = match cookies.get(AUTH_COOKIE_NAME) {
        Some(jwt) => jwt.value(),
        None => {
            return Err(auth_problem("Couldn't extract auth JWT from cookie."));
        }
    };

    match decode::<UserRolesToken>(
        &token,
        &DecodingKey::from_secret(USER_AUTH_KEY),
        &Validation::new(Algorithm::PS256),
    ).map(|data| data.claims) {
        Ok(it) => Ok(it),
        Err(_) => Err(auth_problem("JWT cookie was malformed."))
    }
}

mod jwt_numeric_date {
    // Based on: https://github.com/Keats/jsonwebtoken/blob/master/examples/custom_chrono.rs

    //! Custom serialization of DateTime<Utc> to conform to the JWT spec (RFC 7519 section 2, "Numeric Date")
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    /// Serializes a DateTime<Utc> to a Unix timestamp (milliseconds since 1970/1/1T00:00:00T)
    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let timestamp = date.timestamp();
        serializer.serialize_i64(timestamp)
    }

    /// Attempts to deserialize an i64 and use as a Unix timestamp
    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
        where
            D: Deserializer<'de>,
    {
        Utc.timestamp_opt(i64::deserialize(deserializer)?, 0)
            .single() // If there are multiple or no valid DateTimes from timestamp, return None
            .ok_or_else(|| serde::de::Error::custom("Invalid Unix timestamp value."))
    }
}