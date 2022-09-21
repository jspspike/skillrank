mod matchmaking;
mod rankings;
mod utils;

use rankings::{Match, Player, Session};

use std::collections::HashMap;

use worker::*;

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    utils::set_panic_hook();
    let router = Router::new();

    router
        .on_async("/setup", |req, ctx| async move {
            let namespace = ctx.durable_object("RANKINGS")?;
            let stub = namespace.id_from_name("Spikeball")?.get_stub()?;
            stub.fetch_with_request(req).await
        })
        .on_async("/players", |req, ctx| async move {
            let namespace = ctx.durable_object("RANKINGS")?;
            let stub = namespace.id_from_name("Spikeball")?.get_stub()?;
            stub.fetch_with_request(req).await
        })
        .on_async("/matches", |req, ctx| async move {
            let namespace = ctx.durable_object("RANKINGS")?;
            let stub = namespace.id_from_name("Spikeball")?.get_stub()?;
            stub.fetch_with_request(req).await
        })
        .on_async("/session", |req, ctx| async move {
            let namespace = ctx.durable_object("RANKINGS")?;
            let stub = namespace.id_from_name("Spikeball")?.get_stub()?;
            stub.fetch_with_request(req).await
        })
        .get_async("/", |req, ctx| async move {
            let namespace = ctx.durable_object("RANKINGS")?;
            let stub = namespace.id_from_name("Spikeball")?.get_stub()?;
            //let players_req = Request::new("https://workers.com/players", Method::Get).unwrap();
            //let players: HashMap<u16, Player> = stub.fetch_with_str("https://workers.com/players").await?.json().await?;

            //matchmaking::generate_matches();
            //Response::ok(serde_json::to_string(&players).unwrap())
            Response::ok("")
        })
        .run(req, env)
        .await
}
