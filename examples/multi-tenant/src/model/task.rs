//! Task entity

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "tasks")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub title: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub description: Option<String>,
    pub status: String,
    pub assignee_id: Option<i64>,
    pub due_date: Option<chrono::NaiveDate>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::AssigneeId",
        to = "super::user::Column::Id"
    )]
    Assignee,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Assignee.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

/// Task status values
pub mod status {
    pub const PENDING: &str = "pending";
    pub const IN_PROGRESS: &str = "in_progress";
    pub const COMPLETED: &str = "completed";
    pub const CANCELLED: &str = "cancelled";
}
