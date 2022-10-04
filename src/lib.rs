mod matchmaking;
mod rankings;
mod utils;

use rankings::{make_request, Match, Player, Session};

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
            let mut headers = Headers::new();
            headers.set("content-type", "application/json").unwrap();
            let req = make_request("/players", None, Method::Get).unwrap();
            let players: HashMap<u16, Player> = stub.fetch_with_request(req).await?.json().await?;
            let req = make_request("/session", None, Method::Get).unwrap();
            let session: Session = stub.fetch_with_request(req).await?.json().await?;

            let curr = players.keys().map(|x| *x).collect();

            let game_info = matchmaking::GameInfo {
                games: 2,
                players_per_team: 2,
                stability: 2.0,
            };

            let matches = matchmaking::generate_matches(curr, players, session, game_info)?;
            Response::ok(serde_json::to_string(&matches).unwrap())
            //Response::ok("")
        })
        .run(req, env)
        .await
}
