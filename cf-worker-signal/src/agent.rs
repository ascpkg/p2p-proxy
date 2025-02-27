use std::collections::HashMap;

use worker::*;

use crate::state::{Agent, AppState};

pub async fn handle_pub_agent(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let mut name = String::new();
    if let Some(n) = ctx.param("name") {
        name = n.to_string();
    }

    let mut kv = ctx.kv(AppState::get_kv_store_key())?;
    AppState::insert_or_update_agent(&mut kv, &name, req.json::<Agent>().await?).await;

    Response::from_json(&HashMap::<String, String>::new())
}

pub async fn handle_query_agent(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let mut name = String::new();
    if let Some(n) = ctx.param("name") {
        name = n.to_string();
    }

    let mut kv = ctx.kv(AppState::get_kv_store_key())?;
    let agents = AppState::query_agent(&mut kv, &name).await;

    Response::from_json(&agents)
}
