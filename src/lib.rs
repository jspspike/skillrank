mod games;
mod rankings;
mod utils;

use rankings::{Match, Player, Session};

use std::collections::HashMap;

use skillratings::trueskill::{TrueSkillConfig, TrueSkillRating};
use worker::*;

// Should probably use type parameter for structs where types are used
type RatingType = TrueSkillRating;
type ConfigType = TrueSkillConfig;

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
            let client = rankings::Client::new(ctx, "Spikeball")?;

            let players: HashMap<u16, Player> = client.fetch("/players", None, Method::Get).await?;
            let session: Session = client.fetch("/session", None, Method::Get).await?;

            let curr = players.keys().map(|x| *x).collect();

            let game_info = games::matchmaking::GameInfo {
                games: 2,
                players_per_team: 2,
                stability: 2.0,
            };

            let matches =
                games::matchmaking::generate_matches(curr, players.clone(), session, game_info)?;
            let mut s = String::new();
            for m in matches {
                s.push_str(format!("{:?}\n", players.get(&m.team1[0]).unwrap()).as_str());
                s.push_str(format!("{:?}\n", players.get(&m.team1[1]).unwrap()).as_str());
                s.push_str(format!("{:?}\n", players.get(&m.team2[0]).unwrap()).as_str());
                s.push_str(format!("{:?}\n", players.get(&m.team2[1]).unwrap()).as_str());
                s.push_str("\n\n");
            }

            Response::ok(s)
        })
        .run(req, env)
        .await
}
