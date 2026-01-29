//! GraphQL schema builder

use async_graphql::{Context, EmptySubscription, Object, Result, Schema};
use brylix::prelude::*;
use chrono::NaiveDate;

use super::{LoginResponse, TaskDto, UserDto};
use crate::service::{AuthService, TaskService};

/// Query root
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Get current authenticated user
    async fn me(&self, ctx: &Context<'_>) -> Result<UserDto> {
        let user_id = require_auth_user_id(ctx)?;
        let data = ctx.data_unchecked::<ContextData>();

        let user = AuthService::get_current_user(&data.db, user_id)
            .await
            .map_err(gql_from_domain)?;

        Ok(UserDto::from(user))
    }

    /// Get a task by ID
    async fn task(&self, ctx: &Context<'_>, id: i64) -> Result<TaskDto> {
        let _ = require_auth(ctx)?;
        let data = ctx.data_unchecked::<ContextData>();

        let task = TaskService::get_by_id(&data.db, id)
            .await
            .map_err(gql_from_domain)?;

        Ok(TaskDto::from(task))
    }

    /// List all tasks
    async fn tasks(&self, ctx: &Context<'_>) -> Result<Vec<TaskDto>> {
        let _ = require_auth(ctx)?;
        let data = ctx.data_unchecked::<ContextData>();

        let tasks = TaskService::list(&data.db)
            .await
            .map_err(gql_from_domain)?;

        Ok(tasks.into_iter().map(TaskDto::from).collect())
    }

    /// Get tasks assigned to me
    async fn my_tasks(&self, ctx: &Context<'_>) -> Result<Vec<TaskDto>> {
        let user_id = require_auth_user_id(ctx)?;
        let data = ctx.data_unchecked::<ContextData>();

        let tasks = TaskService::get_my_tasks(&data.db, user_id)
            .await
            .map_err(gql_from_domain)?;

        Ok(tasks.into_iter().map(TaskDto::from).collect())
    }
}

/// Mutation root
pub struct MutationRoot;

#[Object]
impl MutationRoot {
    /// Login and get JWT token
    async fn login(
        &self,
        ctx: &Context<'_>,
        email: String,
        password: String,
    ) -> Result<LoginResponse> {
        let data = ctx.data_unchecked::<ContextData>();
        let config = brylix::config::Config::get();

        let tenant_name = data.tenant_name().ok_or_else(|| {
            gql_bad_request("Tenant context required for login")
        })?;

        let token = AuthService::login(&data.db, config, tenant_name, &email, &password)
            .await
            .map_err(gql_from_domain)?;

        // Decode token to get user ID
        let claims = brylix::auth::validate_jwt(&token)
            .map_err(|_| gql_bad_request("Invalid token generated"))?;

        let user_id: i64 = claims.sub.parse()
            .map_err(|_| gql_bad_request("Invalid user ID in token"))?;

        let user = AuthService::get_current_user(&data.db, user_id)
            .await
            .map_err(gql_from_domain)?;

        Ok(LoginResponse {
            token,
            user: UserDto::from(user),
        })
    }

    /// Register a new user
    async fn register(
        &self,
        ctx: &Context<'_>,
        email: String,
        password: String,
        name: String,
    ) -> Result<UserDto> {
        let data = ctx.data_unchecked::<ContextData>();

        let user = AuthService::register(&data.db, email, password, name)
            .await
            .map_err(gql_from_domain)?;

        Ok(UserDto::from(user))
    }

    /// Create a new task
    async fn create_task(
        &self,
        ctx: &Context<'_>,
        title: String,
        description: Option<String>,
        assignee_id: Option<i64>,
        due_date: Option<NaiveDate>,
    ) -> Result<TaskDto> {
        let _ = require_auth(ctx)?;
        let data = ctx.data_unchecked::<ContextData>();

        let task = TaskService::create(&data.db, title, description, assignee_id, due_date)
            .await
            .map_err(gql_from_domain)?;

        Ok(TaskDto::from(task))
    }

    /// Update a task
    async fn update_task(
        &self,
        ctx: &Context<'_>,
        id: i64,
        title: Option<String>,
        description: Option<String>,
        status: Option<String>,
        assignee_id: Option<i64>,
        due_date: Option<NaiveDate>,
    ) -> Result<TaskDto> {
        let _ = require_auth(ctx)?;
        let data = ctx.data_unchecked::<ContextData>();

        let task = TaskService::update(&data.db, id, title, description, status, assignee_id, due_date)
            .await
            .map_err(gql_from_domain)?;

        Ok(TaskDto::from(task))
    }

    /// Assign task to a user
    async fn assign_task(&self, ctx: &Context<'_>, id: i64, assignee_id: i64) -> Result<TaskDto> {
        let _ = require_auth(ctx)?;
        let data = ctx.data_unchecked::<ContextData>();

        let task = TaskService::assign(&data.db, id, assignee_id)
            .await
            .map_err(gql_from_domain)?;

        Ok(TaskDto::from(task))
    }

    /// Mark task as completed
    async fn complete_task(&self, ctx: &Context<'_>, id: i64) -> Result<TaskDto> {
        let _ = require_auth(ctx)?;
        let data = ctx.data_unchecked::<ContextData>();

        let task = TaskService::complete(&data.db, id)
            .await
            .map_err(gql_from_domain)?;

        Ok(TaskDto::from(task))
    }

    /// Delete a task
    async fn delete_task(&self, ctx: &Context<'_>, id: i64) -> Result<bool> {
        let _ = require_auth(ctx)?;
        let data = ctx.data_unchecked::<ContextData>();

        TaskService::delete(&data.db, id)
            .await
            .map_err(gql_from_domain)?;

        Ok(true)
    }
}

/// Build the GraphQL schema (without data - data added per request)
pub fn build_schema() -> Schema<QueryRoot, MutationRoot, EmptySubscription> {
    Schema::build(QueryRoot, MutationRoot, EmptySubscription).finish()
}
