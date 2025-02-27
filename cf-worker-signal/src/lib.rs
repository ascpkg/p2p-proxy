use std::sync::Arc;

use worker::*;

pub mod agent;
pub mod sdp;
pub mod state;

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    let state = Arc::new(state::AppState::new());

    let router = Router::with_data(state);

    router
        .post_async("/pub/agent/:name/:uuid", agent::handle_pub_agent)
        .get_async("/query/agent/:name", agent::handle_query_agent)
        .post_async("/pub/client/sdp/:uuid", sdp::handle_pub_client_sdp)
        .get_async("/sub/client/sdp/:uuid", sdp::handle_sub_client_sdp)
        .post_async("/pub/agent/sdp/:uuid", sdp::handle_pub_agent_sdp)
        .get_async("/sub/agent/sdp/:uuid", sdp::handle_sub_agent_sdp)
        .run(req, env)
        .await
}
