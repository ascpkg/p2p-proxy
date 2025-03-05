use std::collections::HashMap;

use worker::*;

use crate::state::{AbstractKvStore, AppStateKvStore};

pub async fn handle_publish_client_sdp(
    mut req: Request,
    ctx: RouteContext<()>,
) -> Result<Response> {
    let mut key = String::new();
    if let Some(uuid) = ctx.param("uuid") {
        key = uuid.to_string();
    };

    let mut kv = AppStateKvStore::new(ctx.kv(AppStateKvStore::get_kv_store_key())?);
    kv.insert_or_update_client_sdp(&key, req.json().await?)
        .await;

    Response::from_json(&HashMap::<String, String>::new())
}

pub async fn handle_query_client_sdp(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let mut key = String::new();
    if let Some(uuid) = ctx.param("uuid") {
        key = uuid.to_string();
    };

    let mut kv = AppStateKvStore::new(ctx.kv(AppStateKvStore::get_kv_store_key())?);
    let sdps = kv.query_client_sdp(&key).await;
    Response::from_json(&sdps)
}

pub async fn handle_delete_client_sdp(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let mut key = String::new();
    if let Some(uuid) = ctx.param("uuid") {
        key = uuid.to_string();
    };

    let mut kv = AppStateKvStore::new(ctx.kv(AppStateKvStore::get_kv_store_key())?);
    kv.delete_client_sdp(&key, req.json().await?).await;

    Response::from_json(&HashMap::<String, String>::new())
}

pub async fn handle_publish_agent_sdp(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let mut key = String::new();
    if let Some(uuid) = ctx.param("uuid") {
        key = uuid.to_string();
    };

    let mut kv = AppStateKvStore::new(ctx.kv(AppStateKvStore::get_kv_store_key())?);
    kv.insert_or_update_agent_sdp(&key, req.json().await?).await;

    Response::from_json(&HashMap::<String, String>::new())
}

pub async fn handle_query_agent_sdp(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let mut key = String::new();
    if let Some(uuid) = ctx.param("uuid") {
        key = uuid.to_string();
    };

    let mut kv = AppStateKvStore::new(ctx.kv(AppStateKvStore::get_kv_store_key())?);
    let sdps = kv.query_agent_sdp(&key).await;
    Response::from_json(&sdps)
}

pub async fn handle_delete_agent_sdp(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let mut key = String::new();
    if let Some(uuid) = ctx.param("uuid") {
        key = uuid.to_string();
    };

    let mut kv = AppStateKvStore::new(ctx.kv(AppStateKvStore::get_kv_store_key())?);
    kv.delete_agent_sdp(&key, req.json().await?).await;

    Response::from_json(&HashMap::<String, String>::new())
}
