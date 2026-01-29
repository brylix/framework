//! GraphQL type definitions (DTOs)

use async_graphql::SimpleObject;
use chrono::NaiveDateTime;

use crate::model::post;

/// Post data transfer object for GraphQL
#[derive(Debug, Clone, SimpleObject)]
#[graphql(name = "Post")]
pub struct PostDto {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub published: bool,
    #[graphql(name = "createdAt")]
    pub created_at: NaiveDateTime,
    #[graphql(name = "updatedAt")]
    pub updated_at: Option<NaiveDateTime>,
}

impl From<post::Model> for PostDto {
    fn from(m: post::Model) -> Self {
        Self {
            id: m.id,
            title: m.title,
            content: m.content,
            published: m.published,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}
