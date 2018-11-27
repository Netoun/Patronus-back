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
  pub created_at: NaiveDateTime,
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
        "INSERT INTO Support(user_id, project_id) VALUES ($1, $2, $3)",
        &[&support.user_id, &support.project_id],
      )
      .unwrap();
    support
  }
}

#[derive(Serialize, Deserialize)]
pub struct Project {
  pub project_id: Uuid,
  pub name: String,
  pub description: String,
  pub owner_id: Uuid,
  pub created_at: NaiveDateTime,
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
        "INSERT INTO project (name, description, owner_id) VALUES ($1, $2, $3, $4, $5, $6)",
        &[&project.name, &project.description, &project.owner_id],
      )
      .unwrap();
    project
  }

  pub fn read(connection: &Connection) -> Vec<Project> {
    connection
      .query("SELECT * FROM profile", &[])
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
      .query("SELECT * from profile WHERE uuid=$1", &[&uuid])
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
