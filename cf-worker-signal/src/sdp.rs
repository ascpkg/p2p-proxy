use std::collections::HashMap;

use worker::*;

use crate::state::AppState;

pub async fn handle_pub_client_sdp(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let mut key = String::new();
    if let Some(uuid) = ctx.param("uuid") {
        key = uuid.to_string();
    };

    let mut kv = ctx.kv(AppState::get_kv_store_key())?;
    AppState::insert_or_update_client_sdp(&mut kv, &key, req.json().await?).await;

    Response::from_json(&HashMap::<String, String>::new())
}

pub async fn handle_sub_client_sdp(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let mut key = String::new();
    if let Some(uuid) = ctx.param("uuid") {
        key = uuid.to_string();
    };

    let mut kv = ctx.kv(AppState::get_kv_store_key())?;
    let sdps = AppState::query_client_sdp(&mut kv, &key).await;
    Response::from_json(&sdps)
}

pub async fn handle_pub_agent_sdp(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let mut key = String::new();
    if let Some(uuid) = ctx.param("uuid") {
        key = uuid.to_string();
    };

    let mut kv = ctx.kv(AppState::get_kv_store_key())?;
    AppState::insert_or_update_agent_sdp(&mut kv, &key, req.json().await?).await;

    Response::from_json(&HashMap::<String, String>::new())
}

pub async fn handle_sub_agent_sdp(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let mut key = String::new();
    if let Some(uuid) = ctx.param("uuid") {
        key = uuid.to_string();
    };

    let mut kv = ctx.kv(AppState::get_kv_store_key())?;
    let sdps = AppState::query_agent_sdp(&mut kv, &key).await;
    Response::from_json(&sdps)
}
