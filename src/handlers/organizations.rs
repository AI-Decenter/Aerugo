// src/handlers/organizations.rs - Fixed version
use anyhow::{bail, Context, Result};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sqlx::PgPool;
use validator::Validate;

use crate::{
    models::organizations::{
        AddMemberRequest, CreateOrganizationRequest, Organization, OrganizationMember,
        OrganizationRole, UpdateMemberRequest, UpdateOrganizationRequest,
    },
    AppState,
};

// Create a new organization
pub async fn create_organization(
    State(state): State<AppState>,
    Json(req): Json<CreateOrganizationRequest>,
) -> impl IntoResponse {
    // Validate request
    if let Err(validation_errors) = req.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Validation failed",
                "details": validation_errors
            })),
        );
    }

    // TODO: Get user_id from JWT token when auth middleware is implemented
    let user_id = 1i64; // Placeholder

    match create_org_internal(&state.db_pool, req, user_id).await {
        Ok(organization) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "organization": organization
            })),
        ),
        Err(e) => {
            tracing::error!("Failed to create organization: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": e.to_string()
                })),
            )
        }
    }
}

// Get organization details
pub async fn get_organization(
    State(state): State<AppState>,
    Path(org_name): Path<String>,
) -> impl IntoResponse {
    match get_org_internal(&state.db_pool, &org_name).await {
        Ok(Some(organization)) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "organization": organization
            })),
        ),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Organization not found"
            })),
        ),
        Err(e) => {
            tracing::error!("Failed to get organization: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error"
                })),
            )
        }
    }
}

// Update organization
pub async fn update_organization(
    State(state): State<AppState>,
    Path(org_name): Path<String>,
    Json(req): Json<UpdateOrganizationRequest>,
) -> impl IntoResponse {
    if let Err(validation_errors) = req.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Validation failed",
                "details": validation_errors
            })),
        );
    }

    // TODO: Get user_id from JWT token
    let user_id = 1i64; // Placeholder

    match update_org_internal(&state.db_pool, &org_name, req, user_id).await {
        Ok(organization) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "organization": organization
            })),
        ),
        Err(e) => {
            tracing::error!("Failed to update organization: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": e.to_string()
                })),
            )
        }
    }
}

// Delete organization
pub async fn delete_organization(
    State(state): State<AppState>,
    Path(org_name): Path<String>,
) -> impl IntoResponse {
    // TODO: Get user_id from JWT token
    let user_id = 1i64; // Placeholder

    match delete_org_internal(&state.db_pool, &org_name, user_id).await {
        Ok(_) => (StatusCode::NO_CONTENT, Json(serde_json::json!({}))),
        Err(e) => {
            tracing::error!("Failed to delete organization: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": e.to_string()
                })),
            )
        }
    }
}

// Get organization members
pub async fn get_organization_members(
    State(state): State<AppState>,
    Path(org_name): Path<String>,
) -> impl IntoResponse {
    // TODO: Get user_id from JWT token
    let user_id = Some(1i64); // Placeholder

    match get_members_internal(&state.db_pool, &org_name, user_id).await {
        Ok(members) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "members": members
            })),
        ),
        Err(e) => {
            tracing::error!("Failed to get organization members: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": e.to_string()
                })),
            )
        }
    }
}

// Add member to organization
pub async fn add_organization_member(
    State(state): State<AppState>,
    Path(org_name): Path<String>,
    Json(req): Json<AddMemberRequest>,
) -> impl IntoResponse {
    if let Err(validation_errors) = req.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Validation failed",
                "details": validation_errors
            })),
        );
    }

    // TODO: Get user_id from JWT token
    let inviter_id = 1i64; // Placeholder

    match add_member_internal(&state.db_pool, &org_name, req, inviter_id).await {
        Ok(member) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "member": member
            })),
        ),
        Err(e) => {
            tracing::error!("Failed to add organization member: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": e.to_string()
                })),
            )
        }
    }
}

// Update member role
pub async fn update_member_role(
    State(state): State<AppState>,
    Path((org_name, member_id)): Path<(String, i64)>,
    Json(req): Json<UpdateMemberRequest>,
) -> impl IntoResponse {
    // TODO: Get user_id from JWT token
    let updater_id = 1i64; // Placeholder

    match update_member_role_internal(&state.db_pool, &org_name, member_id, req, updater_id).await {
        Ok(member) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "member": member
            })),
        ),
        Err(e) => {
            tracing::error!("Failed to update member role: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": e.to_string()
                })),
            )
        }
    }
}

// Remove member from organization
pub async fn remove_organization_member(
    State(state): State<AppState>,
    Path((org_name, member_id)): Path<(String, i64)>,
) -> impl IntoResponse {
    // TODO: Get user_id from JWT token
    let remover_id = 1i64; // Placeholder

    match remove_member_internal(&state.db_pool, &org_name, member_id, remover_id).await {
        Ok(_) => (StatusCode::NO_CONTENT, Json(serde_json::json!({}))),
        Err(e) => {
            tracing::error!("Failed to remove organization member: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": e.to_string()
                })),
            )
        }
    }
}

// List user's organizations
pub async fn list_user_organizations(State(state): State<AppState>) -> impl IntoResponse {
    // TODO: Get user_id from JWT token
    let user_id = 1i64; // Placeholder

    match list_user_orgs_internal(&state.db_pool, user_id).await {
        Ok(organizations) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "organizations": organizations
            })),
        ),
        Err(e) => {
            tracing::error!("Failed to list user organizations: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error"
                })),
            )
        }
    }
}

// Helper function to get user's role in organization
async fn get_user_role_in_org(
    pool: &PgPool,
    org_name: &str,
    user_id: i64,
) -> Result<Option<OrganizationRole>> {
    let result = sqlx::query!(
        r#"
        SELECT om.role
        FROM organization_members om
        JOIN organizations o ON om.organization_id = o.id
        WHERE o.name = $1 AND om.user_id = $2
        "#,
        org_name,
        user_id
    )
    .fetch_optional(pool)
    .await?;

    match result {
        Some(row) => match row.role.as_str() {
            "owner" => Ok(Some(OrganizationRole::Owner)),
            "admin" => Ok(Some(OrganizationRole::Admin)),
            "member" => Ok(Some(OrganizationRole::Member)),
            _ => Ok(None),
        },
        None => Ok(None),
    }
}

// Internal database functions
async fn create_org_internal(
    pool: &PgPool,
    req: CreateOrganizationRequest,
    creator_id: i64,
) -> Result<Organization> {
    let mut tx = pool.begin().await?;

    // Check if organization name already exists
    let existing = sqlx::query!("SELECT id FROM organizations WHERE name = $1", req.name)
        .fetch_optional(&mut *tx)
        .await?;

    if existing.is_some() {
        bail!("Organization with name '{}' already exists", req.name);
    }

    // Create organization
    let org = sqlx::query_as!(
        Organization,
        r#"
        INSERT INTO organizations (name, display_name, description, website_url)
        VALUES ($1, $2, $3, $4)
        RETURNING id, name, display_name, description, website_url, avatar_url, created_at, updated_at
        "#,
        req.name,
        req.display_name,
        req.description,
        req.website_url,
    )
    .fetch_one(&mut *tx)
    .await?;

    // Add creator as owner
    sqlx::query!(
        r#"
        INSERT INTO organization_members (organization_id, user_id, role)
        VALUES ($1, $2, $3)
        "#,
        org.id,
        creator_id,
        "owner" // Use string instead of enum
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(org)
}

async fn get_org_internal(pool: &PgPool, org_name: &str) -> Result<Option<Organization>> {
    sqlx::query_as!(
        Organization,
        r#"
        SELECT id, name, display_name, description, website_url, avatar_url, created_at, updated_at
        FROM organizations
        WHERE name = $1
        "#,
        org_name
    )
    .fetch_optional(pool)
    .await
    .context("Failed to fetch organization")
}

async fn update_org_internal(
    pool: &PgPool,
    org_name: &str,
    req: UpdateOrganizationRequest,
    user_id: i64,
) -> Result<Organization> {
    // Check if user has permission to update
    let user_role = get_user_role_in_org(pool, org_name, user_id).await?;
    if !user_role
        .map(|r| r.can_manage_organization())
        .unwrap_or(false)
    {
        bail!("Insufficient permissions to update organization");
    }

    sqlx::query_as!(
        Organization,
        r#"
        UPDATE organizations
        SET 
            display_name = COALESCE($2, display_name),
            description = COALESCE($3, description),
            website_url = COALESCE($4, website_url),
            avatar_url = COALESCE($5, avatar_url),
            updated_at = CURRENT_TIMESTAMP
        WHERE name = $1
        RETURNING id, name, display_name, description, website_url, avatar_url, created_at, updated_at
        "#,
        org_name,
        req.display_name,
        req.description,
        req.website_url,
        req.avatar_url,
    )
    .fetch_one(pool)
    .await
    .context("Organization not found")
}

async fn delete_org_internal(pool: &PgPool, org_name: &str, user_id: i64) -> Result<()> {
    let user_role = get_user_role_in_org(pool, org_name, user_id).await?;
    if !user_role
        .map(|r| r.can_delete_organization())
        .unwrap_or(false)
    {
        bail!("Only organization owners can delete organizations");
    }

    let result = sqlx::query!("DELETE FROM organizations WHERE name = $1", org_name)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        bail!("Organization not found");
    }

    Ok(())
}

async fn get_members_internal(
    pool: &PgPool,
    org_name: &str,
    user_id: Option<i64>,
) -> Result<Vec<OrganizationMember>> {
    // Check if user has access to view members
    if let Some(uid) = user_id {
        let user_role = get_user_role_in_org(pool, org_name, uid).await?;
        if user_role.is_none() {
            bail!("Access denied: not a member of this organization");
        }
    }

    sqlx::query_as!(
        OrganizationMember,
        r#"
        SELECT 
            om.id, om.organization_id, om.user_id, om.role,
            om.joined_at, om.invited_at, om.invited_by,
            u.username, u.email
        FROM organization_members om
        JOIN users u ON om.user_id = u.id
        JOIN organizations o ON om.organization_id = o.id
        WHERE o.name = $1
        ORDER BY om.joined_at ASC
        "#,
        org_name
    )
    .fetch_all(pool)
    .await
    .context("Failed to fetch organization members")
}

async fn add_member_internal(
    pool: &PgPool,
    org_name: &str,
    req: AddMemberRequest,
    inviter_id: i64,
) -> Result<OrganizationMember> {
    let inviter_role = get_user_role_in_org(pool, org_name, inviter_id).await?;
    if !inviter_role
        .map(|r| r.can_manage_members())
        .unwrap_or(false)
    {
        bail!("Insufficient permissions to add members");
    }

    // Get organization ID
    let org = sqlx::query!("SELECT id FROM organizations WHERE name = $1", org_name)
        .fetch_one(pool)
        .await
        .context("Organization not found")?;

    // Find user by email
    let user = sqlx::query!(
        "SELECT id, username, email FROM users WHERE email = $1",
        req.email
    )
    .fetch_one(pool)
    .await
    .context("User not found with that email")?;

    // Check if user is already a member
    let existing = sqlx::query!(
        "SELECT id FROM organization_members WHERE organization_id = $1 AND user_id = $2",
        org.id,
        user.id
    )
    .fetch_optional(pool)
    .await?;

    if existing.is_some() {
        bail!("User is already a member of this organization");
    }

    // Add member
    let member_id = sqlx::query!(
        r#"
        INSERT INTO organization_members (organization_id, user_id, role, invited_by)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
        org.id,
        user.id,
        req.role.to_string(),
        inviter_id,
    )
    .fetch_one(pool)
    .await?
    .id;

    // Return the created member
    let member = OrganizationMember {
        id: member_id,
        organization_id: org.id,
        user_id: user.id,
        role: req.role.to_string(),
        joined_at: chrono::Utc::now(),
        invited_at: Some(chrono::Utc::now()),
        invited_by: Some(inviter_id),
        username: user.username,
        email: user.email,
    };

    Ok(member)
}

async fn update_member_role_internal(
    pool: &PgPool,
    org_name: &str,
    member_user_id: i64,
    req: UpdateMemberRequest,
    updater_id: i64,
) -> Result<OrganizationMember> {
    let updater_role = get_user_role_in_org(pool, org_name, updater_id).await?;
    let target_current_role = get_user_role_in_org(pool, org_name, member_user_id).await?;

    if let (Some(updater), Some(target)) = (updater_role, target_current_role) {
        if !updater.can_change_role_to(&req.role) {
            bail!("Insufficient permissions to assign this role");
        }
        if !updater.can_remove_member(&target) {
            bail!("Insufficient permissions to modify this member");
        }
    } else {
        bail!("Invalid member or insufficient permissions");
    }

    let org = sqlx::query!("SELECT id FROM organizations WHERE name = $1", org_name)
        .fetch_one(pool)
        .await
        .context("Organization not found")?;

    // Update the role
    sqlx::query!(
        "UPDATE organization_members SET role = $3 WHERE organization_id = $1 AND user_id = $2",
        org.id,
        member_user_id,
        req.role.to_string()
    )
    .execute(pool)
    .await?;

    // Fetch and return updated member info
    let member = sqlx::query_as!(
        OrganizationMember,
        r#"
        SELECT 
            om.id, om.organization_id, om.user_id, om.role,
            om.joined_at, om.invited_at, om.invited_by,
            u.username, u.email
        FROM organization_members om
        JOIN users u ON om.user_id = u.id
        WHERE om.organization_id = $1 AND om.user_id = $2
        "#,
        org.id,
        member_user_id
    )
    .fetch_one(pool)
    .await
    .context("Member not found")?;

    Ok(member)
}

async fn remove_member_internal(
    pool: &PgPool,
    org_name: &str,
    member_user_id: i64,
    remover_id: i64,
) -> Result<()> {
    let remover_role = get_user_role_in_org(pool, org_name, remover_id).await?;
    let target_role = get_user_role_in_org(pool, org_name, member_user_id).await?;

    // Allow self-removal for any role
    if remover_id != member_user_id {
        if let (Some(remover), Some(target)) = (remover_role, target_role) {
            if !remover.can_remove_member(&target) {
                bail!("Insufficient permissions to remove this member");
            }
        } else {
            bail!("Invalid member or insufficient permissions");
        }
    }

    let org = sqlx::query!("SELECT id FROM organizations WHERE name = $1", org_name)
        .fetch_one(pool)
        .await
        .context("Organization not found")?;

    let result = sqlx::query!(
        "DELETE FROM organization_members WHERE organization_id = $1 AND user_id = $2",
        org.id,
        member_user_id
    )
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        bail!("Member not found");
    }

    Ok(())
}

async fn list_user_orgs_internal(pool: &PgPool, user_id: i64) -> Result<Vec<Organization>> {
    sqlx::query_as!(
        Organization,
        r#"
        SELECT o.id, o.name, o.display_name, o.description, 
               o.website_url, o.avatar_url, o.created_at, o.updated_at
        FROM organizations o
        JOIN organization_members om ON o.id = om.organization_id
        WHERE om.user_id = $1
        ORDER BY o.name
        "#,
        user_id
    )
    .fetch_all(pool)
    .await
    .context("Failed to fetch user organizations")
}
