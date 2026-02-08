//! GraphQL schema builder

use async_graphql::{Context, EmptySubscription, Object, Result, Schema};
use brylix::prelude::*;
use sea_orm::DatabaseConnection;

use super::PostDto;
use crate::service::PostService;

/// Query root
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Get a post by ID
    async fn post(&self, ctx: &Context<'_>, id: i64) -> Result<PostDto> {
        let data = ctx.data_unchecked::<ContextData>();
        let post = PostService::get_by_id(&data.db, id)
            .await
            .map_err(gql_from_domain)?;
        Ok(PostDto::from(post))
    }

    /// List all posts
    async fn posts(&self, ctx: &Context<'_>) -> Result<Vec<PostDto>> {
        let data = ctx.data_unchecked::<ContextData>();
        let posts = PostService::list(&data.db)
            .await
            .map_err(gql_from_domain)?;
        Ok(posts.into_iter().map(PostDto::from).collect())
    }
}

/// Mutation root
pub struct MutationRoot;

#[Object]
impl MutationRoot {
    /// Create a new post
    async fn create_post(
        &self,
        ctx: &Context<'_>,
        title: String,
        content: String,
    ) -> Result<PostDto> {
        let data = ctx.data_unchecked::<ContextData>();
        let post = PostService::create(&data.db, title, content)
            .await
            .map_err(gql_from_domain)?;
        Ok(PostDto::from(post))
    }

    /// Update a post
    async fn update_post(
        &self,
        ctx: &Context<'_>,
        id: i64,
        title: Option<String>,
        content: Option<String>,
    ) -> Result<PostDto> {
        let data = ctx.data_unchecked::<ContextData>();
        let post = PostService::update(&data.db, id, title, content, None)
            .await
            .map_err(gql_from_domain)?;
        Ok(PostDto::from(post))
    }

    /// Publish a post
    async fn publish_post(&self, ctx: &Context<'_>, id: i64) -> Result<PostDto> {
        let data = ctx.data_unchecked::<ContextData>();
        let post = PostService::publish(&data.db, id)
            .await
            .map_err(gql_from_domain)?;
        Ok(PostDto::from(post))
    }

    /// Delete a post
    async fn delete_post(&self, ctx: &Context<'_>, id: i64) -> Result<bool> {
        let data = ctx.data_unchecked::<ContextData>();
        PostService::delete(&data.db, id)
            .await
            .map_err(gql_from_domain)?;
        Ok(true)
    }
}

/// Build the GraphQL schema
pub fn build_schema(db: DatabaseConnection) -> Schema<QueryRoot, MutationRoot, EmptySubscription> {
    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(ContextData::single_tenant(db, None, None))
        .finish()
}
