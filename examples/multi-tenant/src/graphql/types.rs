//! GraphQL type definitions (DTOs)

use async_graphql::SimpleObject;
use chrono::{NaiveDate, NaiveDateTime};

use crate::model::{task, user};

/// User data transfer object
#[derive(Debug, Clone, SimpleObject)]
#[graphql(name = "User")]
pub struct UserDto {
    pub id: i64,
    pub email: String,
    pub name: String,
    #[graphql(name = "createdAt")]
    pub created_at: NaiveDateTime,
}

impl From<user::Model> for UserDto {
    fn from(m: user::Model) -> Self {
        Self {
            id: m.id,
            email: m.email,
            name: m.name,
            created_at: m.created_at,
        }
    }
}

/// Task data transfer object
#[derive(Debug, Clone, SimpleObject)]
#[graphql(name = "Task")]
pub struct TaskDto {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    #[graphql(name = "assigneeId")]
    pub assignee_id: Option<i64>,
    #[graphql(name = "dueDate")]
    pub due_date: Option<NaiveDate>,
    #[graphql(name = "createdAt")]
    pub created_at: NaiveDateTime,
    #[graphql(name = "updatedAt")]
    pub updated_at: Option<NaiveDateTime>,
}

impl From<task::Model> for TaskDto {
    fn from(m: task::Model) -> Self {
        Self {
            id: m.id,
            title: m.title,
            description: m.description,
            status: m.status,
            assignee_id: m.assignee_id,
            due_date: m.due_date,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

/// Login response
#[derive(Debug, Clone, SimpleObject)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserDto,
}
