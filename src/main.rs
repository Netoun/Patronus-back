#![feature(plugin)]
#![feature(proc_macro_hygiene, decl_macro)]
extern crate jwt;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_cors;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;
extern crate bcrypt;
extern crate chrono;
extern crate crypto;
extern crate postgres;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate uuid;
use bcrypt::hash;
use rocket::http::Method;
use rocket::http::Status;
use rocket_contrib::json::{Json, JsonValue};
use rocket_cors::{AllowedHeaders, AllowedOrigins};

mod db;
mod project;
mod token;
mod user;

use crypto::sha2::Sha256;
use jwt::{Header, Registered, Token};
use project::newProject;
use project::newSupport;
use project::Project;
use project::Support;
use token::AdminToken;
use token::ApiToken;
use user::newSub;
use user::newUser;
use user::Sub;
use user::User;
use user::UserWithPassword;
use uuid::Uuid;

// -------------
// --CODE USER--
// -------------
#[post("/", data = "<user>", format = "application/json")]
fn create(user: Json<newUser>, connection: db::DbConn) -> Json<newUser> {
  let insert = newUser {
    full_name: user.full_name.to_owned(),
    email: user.email.to_owned(),
    password: hash(&user.password, 4).unwrap(),
    role: 1,
  };
  Json(User::create(insert, &connection))
}

#[get("/profile")]
fn profile(key: ApiToken, connection: db::DbConn) -> Result<JsonValue, Status> {
  User::get_user(key.0, &connection)
    .map(|user| json!(user))
    .map_err(|error| convert_auth_error(error))
}

#[get("/profile", rank = 2)]
fn profile_error() -> JsonValue {
  json!(
        {
            "success": false,
            "message": "Not authorized"
        }
    )
}

#[get("/")]
fn read(connection: db::DbConn) -> JsonValue {
  json!(User::read(&connection))
}

#[get("/count")]
fn read_count(connection: db::DbConn) -> JsonValue {
  json!(User::read(&connection).len())
}

#[post("/", data = "<user>", format = "application/json")]
fn login(user: Json<UserWithPassword>, connection: db::DbConn) -> Result<JsonValue, Status> {
  let header: Header = Default::default();
  let auth = UserWithPassword {
    email: user.email.to_owned(),
    password: user.password.to_owned(),
  };
  match User::check_login(auth, &connection) {
    None => Err(Status::NotFound),
    Some(user) => {
      let claims = Registered {
        sub: Some(user.uuid.to_string().into()),
        aud: Some(user.role.to_string().into()),
        ..Default::default()
      };
      Token::new(header, claims)
        .signed(b"secret_key", Sha256::new())
        .map(|message| json!({ "success": true, "token": message }))
        .map_err(|_| Status::InternalServerError)
    }
  }
}

// ----------------
// --CODE PROJECT--
// ----------------

#[post("/", data = "<project>", format = "application/json")]
fn create_project(project: Json<newProject>, connection: db::DbConn) -> Json<newProject> {
  let insert = newProject {
    name: project.name.to_owned(),
    description: project.description.to_owned(),
    owner_id: project.owner_id.to_owned(),
    image_url: project.image_url.to_owned(),
  };
  Json(Project::create(insert, &connection))
}

#[get("/<uuid>")]
fn project_info(uuid: String, connection: db::DbConn) -> Result<JsonValue, Status> {
  Project::get_project(uuid, &connection)
    .map(|user| json!(user))
    .map_err(|error| convert_auth_error_project(error))
}

#[get("/")]
fn read_project(connection: db::DbConn) -> JsonValue {
  json!(Project::read(&connection))
}

// ----------------
// --CODE SUPPORT--
// ----------------

#[post("/", data = "<vote>", format = "application/json")]
fn voter(vote: Json<newSupport>, connection: db::DbConn) -> Json<newSupport> {
  let insert = newSupport {
    user_id: vote.user_id.to_owned(),
    project_id: vote.project_id.to_owned(),
  };
  Json(Support::create(insert, &connection))
}

#[get("/")]
fn support_user(key: ApiToken, connection: db::DbConn) -> Result<JsonValue, Status> {
  Support::get_support_user(key.0, &connection)
    .map(|support| json!(support))
    .map_err(|error| convert_auth_error_project(error))
}

// ----------------
// --CODE SUB--
// ----------------

#[post("/sub", data = "<sub>", format = "application/json")]
fn new_sub(sub: Json<newSub>, connection: db::DbConn) -> Json<newSub> {
  let insert = newSub {
    user_id: sub.user_id.to_owned(),
    values: sub.values.to_owned(),
  };
  Json(Sub::create(insert, &connection))
}

#[get("/totalsubs")]
fn total_sub(connection: db::DbConn) -> Result<JsonValue, Status> {
  Sub::get_sum(&connection)
    .map(|sub| json!(sub))
    .map_err(|error| convert_auth_error(error))
}

// ----------------
// --CODE ROCKET--
// ----------------

pub fn options() -> rocket_cors::Cors {
  rocket_cors::Cors {
    allowed_origins: AllowedOrigins::all(),
    allowed_methods: vec![Method::Post, Method::Get]
      .into_iter()
      .map(From::from)
      .collect(),
    allowed_headers: AllowedHeaders::all(),
    allow_credentials: true,
    ..Default::default()
  }
}

pub fn convert_auth_error(err: user::AuthenticationError) -> Status {
  use user::AuthenticationError::*;

  match err {
    IncorrectMail => Status::new(404, "Not Mail"),
    IncorrectUuid => Status::new(404, "Not Uuid"),
    IncorrectPassword => Status::new(401, "Unauthorized"),
    BcryptError(e) => Status::new(501, "Internal Server Error"),
    DatabaseError(e) => Status::new(503, "Service Unavailable"),
  }
}

pub fn convert_auth_error_project(err: project::AuthenticationError) -> Status {
  use project::AuthenticationError::*;

  match err {
    IncorrectUuid => Status::new(404, "No Uuid"),
    IncorrectPassword => Status::new(401, "Unauthorized"),
    DatabaseError(e) => Status::new(503, "Service Unavailable"),
  }
}

fn main() {
  let conn = db::connect();

  rocket::ignite()
    .manage(conn)
    .attach(options())
    .mount("/project", routes![create_project, project_info])
    .mount("/projects", routes![read_project])
    .mount("/voter", routes![voter])
    .mount("/support", routes![support_user])
    .mount("/user", routes![create, profile, profile_error, new_sub])
    .mount("/users", routes![read, read_count, total_sub])
    .mount("/login", routes![login])
    .launch();
}

// update, delete
