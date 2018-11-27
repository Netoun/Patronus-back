use rocket::request::{self, FromRequest, Request};
use rocket::Outcome;
use uuid::Uuid;

pub extern crate crypto;
pub extern crate jwt;
pub extern crate rustc_serialize;

use self::crypto::sha2::Sha256;
use self::jwt::{Header, Registered, Token};

pub struct ApiToken(pub String);
pub struct AdminToken(pub String);

pub fn read_token(key: &str) -> Result<String, String> {
  let token =
    Token::<Header, Registered>::parse(key).map_err(|_| "Unable to parse key".to_string())?;

  if token.verify(b"secret_key", Sha256::new()) {
    token.claims.sub.ok_or("Claims not valid".to_string())
  } else {
    Err("Token not valid".to_string())
  }
}

pub fn read_token_admin(key: &str) -> Result<String, String> {
  let token =
    Token::<Header, Registered>::parse(key).map_err(|_| "Unable to parse key".to_string())?;

  if token.verify(b"secret_key", Sha256::new()) && token.claims.aud == Some("2".to_string()) {
    token.claims.sub.ok_or("Claims not valid".to_string())
  } else {
    Err("Token not valid".to_string())
  }
}

impl<'a, 'r> FromRequest<'a, 'r> for ApiToken {
  type Error = ();

  fn from_request(request: &'a Request<'r>) -> request::Outcome<ApiToken, ()> {
    let keys: Vec<_> = request.headers().get("Authentication").collect();
    if keys.len() != 1 {
      return Outcome::Forward(());
    }
    let role = read_token(keys[0]);
    println!("{:?}", role);
    match read_token(keys[0]) {
      Ok(claim) => Outcome::Success(ApiToken(claim)),
      Err(_) => Outcome::Forward(()),
    }
  }
}

impl<'a, 'r> FromRequest<'a, 'r> for AdminToken {
  type Error = ();

  fn from_request(request: &'a Request<'r>) -> request::Outcome<AdminToken, ()> {
    let keys: Vec<_> = request.headers().get("Authentication").collect();
    if keys.len() != 1 {
      return Outcome::Forward(());
    }

    match read_token_admin(keys[0]) {
      Ok(claim) => {
        println!("{}", claim);
        Outcome::Success(AdminToken(claim))
      }
      Err(_) => Outcome::Forward(()),
    }
  }
}
