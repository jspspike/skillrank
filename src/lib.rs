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
            let mut headers = Headers::new();
            headers.set("content-type", "application/json").unwrap();
            let players_req = Request::new_with_init(
                "https://w/players",
                &RequestInit {
                    body: None,
                    headers,
                    cf: CfProperties::default(),
                    method: Method::Get,
                    redirect: RequestRedirect::Follow,
                },
            )
            .unwrap();
            let players: HashMap<u16, Player> =
                stub.fetch_with_request(players_req).await?.json().await?;

            //matchmaking::generate_matches();
            Response::ok(serde_json::to_string(&players).unwrap())
            //Response::ok("")
        })
        .run(req, env)
        .await
}
