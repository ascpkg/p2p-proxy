use std::sync::Arc;

use chrono::Utc;
use worker::*;

use crate::state::{AppState, Sdp};

pub async fn handle_pub_client_sdp(
    mut req: Request,
    ctx: RouteContext<Arc<AppState>>,
) -> Result<Response> {
    let state = ctx.data.clone();

    let mut key = String::new();
    if let Some(uuid) = ctx.param("uuid") {
        key = uuid.to_string();
    };

    let value = Sdp {
        sdp: req.text().await?,
        created_at: Utc::now().timestamp_millis() as u64,
    };
    state.client_sdps.write().unwrap().insert(key, value);

    Response::ok("handle_pub_client_sdp success")
}

pub async fn handle_sub_client_sdp(
    _req: Request,
    ctx: RouteContext<Arc<AppState>>,
) -> Result<Response> {
    let state = ctx.data.clone();

    let mut key = String::new();
    if let Some(uuid) = ctx.param("uuid") {
        key = uuid.to_string();
    };

    let value = state.client_sdps.read().unwrap().get(&key).cloned();
    if let Some(sdp) = value {
        Response::from_json(&sdp)
    } else {
        Response::error("handle_sub_client_sdp error", 404)
    }
}

pub async fn handle_pub_agent_sdp(
    mut req: Request,
    ctx: RouteContext<Arc<AppState>>,
) -> Result<Response> {
    let state = ctx.data.clone();

    let mut key = String::new();
    if let Some(uuid) = ctx.param("uuid") {
        key = uuid.to_string();
    };

    let value = Sdp {
        sdp: req.text().await?,
        created_at: Utc::now().timestamp_millis() as u64,
    };

    state.agent_sdps.write().unwrap().insert(key, value);

    Response::ok("handle_pub_agent_sdp success")
}

pub async fn handle_sub_agent_sdp(
    _req: Request,
    ctx: RouteContext<Arc<AppState>>,
) -> Result<Response> {
    let state = ctx.data.clone();

    let mut key = String::new();
    if let Some(uuid) = ctx.param("uuid") {
        key = uuid.to_string();
    };

    let value = state.agent_sdps.read().unwrap().get(&key).cloned();
    if let Some(sdp) = value {
        Response::from_json(&sdp)
    } else {
        Response::error("handle_sub_agent_sdp error", 404)
    }
}
