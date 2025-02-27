use std::{collections::HashMap, sync::Arc};

use chrono::Utc;
use worker::*;

use crate::state::{Agent, AppState};

pub async fn handle_pub_agent(
    mut req: Request,
    ctx: RouteContext<Arc<AppState>>,
) -> Result<Response> {
    let state = ctx.data.clone();
    let mut agent = req.json::<Agent>().await?;
    agent.last_seen = Utc::now().timestamp_millis() as u64;

    state
        .agents
        .write()
        .unwrap()
        .entry(agent.name.clone())
        .or_insert(HashMap::new())
        .insert(agent.uuid.clone(), agent);

    Response::ok("handle_pub_agent success")
}

pub async fn handle_query_agent(
    _req: Request,
    ctx: RouteContext<Arc<AppState>>,
) -> Result<Response> {
    let state = ctx.data.clone();
    let mut name = String::new();
    if let Some(n) = ctx.param("name") {
        name = n.to_string();
    }

    let agents: Vec<Agent> = if !name.is_empty() {
        // get all agents for a specific name
        state
            .agents
            .read()
            .unwrap()
            .get(&name)
            .unwrap_or(&HashMap::new())
            .values()
            .cloned()
            .collect()
    } else {
        // get all agents
        state
            .agents
            .read()
            .unwrap()
            .values()
            .flat_map(|agents| agents.values())
            .cloned()
            .collect()
    };

    Response::from_json(&agents)
}
