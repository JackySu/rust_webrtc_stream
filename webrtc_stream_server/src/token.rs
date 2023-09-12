use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::user::User;
use crate::AUTHORIZATION_TOKEN_EXPIRACY_TIME;

#[derive(Serialize, Deserialize)]
pub struct AuthorizationToken {
    pub uid: String,
    pub exp: i64,
}

impl From<User> for AuthorizationToken {
    fn from(user: User) -> AuthorizationToken {
        AuthorizationToken {
            uid: user.id,
            exp: (chrono::Utc::now() + *AUTHORIZATION_TOKEN_EXPIRACY_TIME).timestamp(),
        }
    }
}

pub trait SerializableToken
where
    Self: Serialize + Sized + for<'a> Deserialize<'a>,
{
    fn encode(&self, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
        encode(
            &Header::default(),
            &self,
            &EncodingKey::from_secret(secret.as_ref()),
        )
    }

    fn decode(token: &str, secret: &str) -> Result<Self, jsonwebtoken::errors::Error> {
        let token = decode::<Self>(
            token,
            &DecodingKey::from_secret(secret.as_ref()),
            &Validation::new(Algorithm::HS256),
        )?;
        Ok(token.claims)
    }
}

impl SerializableToken for AuthorizationToken {}