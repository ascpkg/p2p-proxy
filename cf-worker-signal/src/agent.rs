use std::collections::HashMap;

use worker::*;

use crate::state::{AbstractKvStore, Agent, AppStateKvStore};

pub async fn handle_publish_agent(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let mut name = String::new();
    if let Some(n) = ctx.param("name") {
        name = n.to_string();
    }

    let mut kv = AppStateKvStore::new(ctx.kv(AppStateKvStore::get_kv_store_key())?);
    kv.insert_or_update_agent(&name, req.json::<Agent>().await?)
        .await;

    Response::from_json(&HashMap::<String, String>::new())
}

pub async fn handle_query_agent(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let mut name = String::new();
    if let Some(n) = ctx.param("name") {
        name = n.to_string();
    }

    let mut kv = AppStateKvStore::new(ctx.kv(AppStateKvStore::get_kv_store_key())?);
    let agents = kv.query_agent(&name).await;

    Response::from_json(&agents)
}
