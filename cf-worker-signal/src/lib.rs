use worker::*;

pub mod agent;
pub mod sdp;
pub mod state;

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    let router = Router::new();

    router
        .post_async("/publish/agent/:name", agent::handle_publish_agent)
        .get_async("/query/agent/:name", agent::handle_query_agent)
        .post_async("/delete/agent/:name", agent::handle_delete_agent)
        .post_async("/publish/client/sdp/:uuid", sdp::handle_publish_client_sdp)
        .get_async("/query/client/sdp/:uuid", sdp::handle_query_client_sdp)
        .post_async("/delete/client/sdp/:uuid", sdp::handle_delete_client_sdp)
        .post_async("/publish/agent/sdp/:uuid", sdp::handle_publish_agent_sdp)
        .get_async("/query/agent/sdp/:uuid", sdp::handle_query_agent_sdp)
        .post_async("/delete/agent/sdp/:uuid", sdp::handle_delete_agent_sdp)
        .run(req, env)
        .await
}
