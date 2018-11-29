use bcrypt::*;
use chrono::prelude::*;
use postgres::rows::Row;
use postgres::Connection;
use uuid::Uuid;

#[derive(Debug)]
pub enum AuthenticationError {
  IncorrectUuid,
  IncorrectPassword,
  IncorrectMail,
  BcryptError(BcryptError),
  DatabaseError(postgres::Error),
}

pub use self::AuthenticationError::IncorrectPassword;

impl From<BcryptError> for AuthenticationError {
  fn from(e: BcryptError) -> Self {
    AuthenticationError::BcryptError(e)
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct newSub {
  pub user_id: Uuid,
  pub values: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Sub {
  pub subscription_id: Uuid,
  pub user_id: Uuid,
  pub values: f64,
  pub created_at: DateTime<Local>,
}

impl Sub {
  pub fn create(sub: newSub, connection: &Connection) -> newSub {
    connection
      .execute(
        r#"INSERT INTO "SUBSCRIPTION" (full_name, values) VALUES ($1, $2)"#,
        &[&sub.user_id, &sub.values],
      )
      .unwrap();
    sub
  }

  pub fn get_sum(connection: &Connection) -> Result<f64, AuthenticationError> {
    println!("ezez");
    let qrystr = format!(r#"SELECT SUM(value) from "SUBSCRIPTION""#);
    let sumSub = connection
      .query(&qrystr, &[])
      .map_err(AuthenticationError::DatabaseError)?;;
    println!("{:?}", sumSub);
    if !sumSub.is_empty() && sumSub.len() == 1 {
      let row = sumSub.get(0);
      let sub_results = row.get(0);
      Ok(sub_results)
    } else {
      Err(AuthenticationError::IncorrectUuid)
    }
  }

  pub fn get_sub(uuid: String, connection: &Connection) -> Result<Sub, AuthenticationError> {
    println!("{:?}", uuid);
    let qrystr = format!(r#"SELECT * from "SUBSCRIPTION" WHERE user_id = '{}'"#, uuid);
    let user = connection
      .query(&qrystr, &[])
      .map_err(AuthenticationError::DatabaseError)?;;
    println!("{:?}", user);
    if !user.is_empty() && user.len() == 1 {
      let row = user.get(0);
      let sub_results = Sub {
        subscription_id: row.get(0),
        user_id: row.get(1),
        values: row.get(2),
        created_at: row.get(3),
      };
      Ok(sub_results)
    } else {
      Err(AuthenticationError::IncorrectUuid)
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct newUser {
  pub full_name: String,
  pub email: String,
  pub password: String,
  pub role: i16,
}

#[derive(Serialize, Deserialize)]
pub struct User {
  pub uuid: Uuid,
  pub full_name: String,
  pub email: String,
  pub created_at: DateTime<Local>,
  pub password: String,
  pub role: i16,
}

#[derive(Serialize, Deserialize)]
pub struct UserWithPassword {
  pub email: String,
  pub password: String,
}

impl User {
  pub fn create(user: newUser, connection: &Connection) -> newUser {
    println!("{:?}", user);
    connection
      .execute(
        r#"INSERT INTO "USER" (full_name, email, password, role) VALUES ($1, $2, $3, $4)"#,
        &[&user.full_name, &user.email, &user.password, &user.role],
      )
      .unwrap();
    user
  }

  pub fn read(connection: &Connection) -> Vec<User> {
    connection
      .query(r#"SELECT * FROM "USER""#, &[])
      .unwrap()
      .into_iter()
      .map(|row| User {
        uuid: row.get(0),
        full_name: row.get(1),
        email: row.get(2),
        created_at: row.get(3),
        password: row.get(4),
        role: row.get(5),
      })
      .collect::<Vec<_>>()
  }

  pub fn get_user(uuid: String, connection: &Connection) -> Result<User, AuthenticationError> {
    println!("{:?}", uuid);
    let qrystr = format!(r#"SELECT * from "USER" WHERE user_id = '{}'"#, uuid);
    let user = connection
      .query(&qrystr, &[])
      .map_err(AuthenticationError::DatabaseError)?;;
    println!("{:?}", user);
    if !user.is_empty() && user.len() == 1 {
      let row = user.get(0);
      let user_results = User {
        uuid: row.get(0),
        full_name: row.get(1),
        email: row.get(2),
        created_at: row.get(3),
        password: row.get(4),
        role: row.get(5),
      };
      Ok(user_results)
    } else {
      Err(AuthenticationError::IncorrectUuid)
    }
  }

  pub fn find_user(
    user: UserWithPassword,
    connection: &Connection,
  ) -> Result<Option<User>, AuthenticationError> {
    let user_and_password = connection
      .query(r#"SELECT * from "USER" WHERE email=$1"#, &[&user.email])
      .map_err(AuthenticationError::DatabaseError)?;
    if !user_and_password.is_empty() && user_and_password.len() == 1 {
      let row = user_and_password.get(0);
      let user_results = User {
        uuid: row.get(0),
        full_name: row.get(1),
        email: row.get(2),
        created_at: row.get(3),
        password: row.get(4),
        role: row.get(5),
      };
      if verify(&user.password, &user_results.password[..])? {
        Ok(Some(user_results))
      } else {
        Err(AuthenticationError::IncorrectPassword)
      }
    } else {
      Err(AuthenticationError::IncorrectMail)
    }
  }

  pub fn check_login(user: UserWithPassword, connection: &Connection) -> Option<User> {
    match User::find_user(user, connection) {
      Ok(user) => user,
      Err(e) => None,
    }
  }
}
