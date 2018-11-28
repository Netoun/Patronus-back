use chrono::prelude::*;
use postgres::rows::Row;
use postgres::Connection;
use uuid::Uuid;

#[derive(Debug)]
pub enum AuthenticationError {
  DatabaseError(postgres::Error),
  IncorrectUuid,
}

#[derive(Serialize, Deserialize)]
pub struct Support {
  pub support_id: Uuid,
  pub user_id: Uuid,
  pub project_id: Uuid,
  pub created_at: DateTime<Local>,
}

#[derive(Serialize, Deserialize)]
pub struct newSupport {
  pub user_id: Uuid,
  pub project_id: Uuid,
}

impl Support {
  pub fn create(support: newSupport, connection: &Connection) -> newSupport {
    connection
      .execute(
        r#"INSERT INTO "SUPPORT"(user_id, project_id) VALUES ($1, $2)"#,
        &[&support.user_id, &support.project_id],
      )
      .unwrap();
    support
  }

  pub fn get_support_user(
    uuid: String,
    connection: &Connection,
  ) -> Result<Support, AuthenticationError> {
    let qrystr = format!(r#"SELECT * from "SUPPORT" WHERE user_id = '{}'"#, uuid);
    let support = connection
      .query(&qrystr, &[])
      .map_err(AuthenticationError::DatabaseError)?;;
    if !support.is_empty() && support.len() == 1 {
      let row = support.get(0);
      let support_results = Support {
        support_id: row.get(0),
        user_id: row.get(1),
        project_id: row.get(2),
        created_at: row.get(3),
      };
      Ok(support_results)
    } else {
      Err(AuthenticationError::IncorrectUuid)
    }
  }
}

#[derive(Serialize, Deserialize)]
pub struct Project {
  pub project_id: Uuid,
  pub name: String,
  pub description: String,
  pub owner_id: Uuid,
  pub created_at: DateTime<Local>,
  pub count: i32,
}

#[derive(Serialize, Deserialize)]
pub struct newProject {
  pub name: String,
  pub description: String,
  pub owner_id: Uuid,
}

impl Project {
  pub fn create(project: newProject, connection: &Connection) -> newProject {
    connection
      .execute(
        r#"INSERT INTO "PROJECT" (name, description, owner_id) VALUES ($1, $2, $3)"#,
        &[&project.name, &project.description, &project.owner_id],
      )
      .unwrap();
    project
  }

  pub fn read(connection: &Connection) -> Vec<Project> {
    connection
      .query(r#"SELECT * FROM "PROJECT""#, &[])
      .unwrap()
      .into_iter()
      .map(|row| Project {
        project_id: row.get(0),
        name: row.get(1),
        description: row.get(2),
        owner_id: row.get(3),
        created_at: row.get(4),
      })
      .collect::<Vec<_>>()
  }

  pub fn get_project(
    uuid: String,
    connection: &Connection,
  ) -> Result<Project, AuthenticationError> {
    let Project = connection
      .query(r#"SELECT p.project_id, p.name, p.description, p.owner_id, p.created_at COUNT(s.support_id) from "PROJECT" as p WHERE uuid=$1"#, &[&uuid])
      .map_err(AuthenticationError::DatabaseError)?;

    if !Project.is_empty() && Project.len() == 1 {
      let row = Project.get(0);
      let Project_results = Project {
        project_id: row.get(0),
        name: row.get(1),
        description: row.get(2),
        owner_id: row.get(3),
        created_at: row.get(4),
      };
      Ok(Project_results)
    } else {
      Err(AuthenticationError::IncorrectUuid)
    }
  }
}
