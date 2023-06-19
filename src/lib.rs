mod games;
mod rankings;
mod scripts;
mod utils;

use games::matchmaking;
use rankings::{Empty, Match, Player, Session};

use std::collections::HashMap;

use serde::Serialize;
use skillratings::trueskill::{TrueSkill, TrueSkillConfig, TrueSkillRating};
use skillratings::{Rating, TeamRatingSystem};
use tinytemplate::TinyTemplate;
use worker::*;

// Should probably use type parameter for structs where types are used
type RatingType = TrueSkillRating;

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    utils::set_panic_hook();
    let router = Router::new();

    router
        .on_async("/:id/players", |req, ctx| async move {
            let id = ctx.param("id").unwrap();
            let namespace = ctx.durable_object("RANKINGS")?;

            let client = rankings::Client::new(&ctx, id)?;
            if !client.check_pass(&req).await? {
                return Response::error("", 401);
            }
            let stub = namespace.id_from_name(id)?.get_stub()?;
            stub.fetch_with_request(req).await
        })
        .on_async("/:id/matches", |req, ctx| async move {
            let id = ctx.param("id").unwrap();
            let namespace = ctx.durable_object("RANKINGS")?;

            let client = rankings::Client::new(&ctx, id)?;
            if !client.check_pass(&req).await? {
                return Response::error("", 401);
            }
            let stub = namespace.id_from_name(id)?.get_stub()?;
            stub.fetch_with_request(req).await
        })
        .on_async("/:id/session", |req, ctx| async move {
            let id = ctx.param("id").unwrap();
            let namespace = ctx.durable_object("RANKINGS")?;

            let client = rankings::Client::new(&ctx, id)?;
            if !client.check_pass(&req).await? {
                return Response::error("", 401);
            }
            let stub = namespace.id_from_name(id)?.get_stub()?;
            stub.fetch_with_request(req).await
        })
        .on_async("/create/:id", |mut req, ctx| async move {
            let id = ctx.param("id").unwrap();
            let client = rankings::Client::new(&ctx, id)?;
            let pass_created: bool = client.fetch("/pass", "", Method::Get).await?;

            if pass_created {
                return Response::error("ID already exists", 406);
            }

            let pass: String = req.text().await?;
            let _: Empty = client.fetch("/setup", &pass, Method::Put).await?;

            ctx.kv("SKILLRANK_IDS")?
                .put(id, Date::now().to_string())?
                .execute()
                .await?;
            Response::ok("")
        })
        .on_async("/:id/generate-matches", |mut req, ctx| async move {
            let id = ctx.param("id").unwrap();
            let client = rankings::Client::new(&ctx, id)?;
            let participants: Vec<u16> = req.json().await?;
            let players: HashMap<u16, Player<RatingType>> =
                client.fetch("/players", "", Method::Get).await?;
            let session: Option<Session> = client.fetch("/session", "", Method::Get).await?;
            let sesh = session.unwrap();
            let game_info = sesh.game_info;

            let matches = matchmaking::generate_matches(participants, &players, sesh, game_info)?;
            let matches_str: String = matches.iter().fold("".to_string(), |acc, m| {
                let team_1 = m.team1.iter().fold("".to_string(), |team1_acc, p| {
                    let player = players.get(p).unwrap();
                    format!("{}{}, ", team1_acc, player.name)
                });
                let team_2 = m.team2.iter().fold("".to_string(), |team2_acc, p| {
                    let player = players.get(p).unwrap();
                    format!("{}{}, ", team2_acc, player.name)
                });

                format!(
                    "{}Game {}<br>Team 1:{}<br>Team 2:{}<br><br>",
                    acc,
                    m.id + 1,
                    team_1,
                    team_2
                )
            });
            Response::ok(matches_str)
        })
        .on_async("/:id/add-match", |mut req, ctx| async move {
            let id = ctx.param("id").unwrap();
            let client = rankings::Client::new(&ctx, id)?;
            if !client.check_pass(&req).await? {
                return Response::error("", 401);
            }

            let m: Match = req.json().await?;
            let rating_system = TrueSkill::new(TrueSkillConfig {
                draw_probability: 0.0,
                beta: 6.0,
                default_dynamics: 0.13,
            });

            games::add_match(&m.team1, &m.team2, client, rating_system).await?;
            Response::ok("")
        })
        .get_async("/:id/player", |_req, ctx| async move {
            let template = include_str!("../content/player.html");
            let mut tt = TinyTemplate::new();
            tt.add_template("/player", template)
                .map_err(|err| err.to_string())?;

            let id = ctx.param("id").unwrap();

            #[derive(Serialize)]
            struct Context {
                id: String,
                default_score: f64,
            }

            let context = Context {
                id: id.clone(),
                default_score: 25.0,
            };

            let mut rendered = tt
                .render("/player", &context)
                .map_err(|err| err.to_string())?;
            rendered.push_str(scripts::PLAYER);
            Response::from_html(rendered)
        })
        .get_async("/:id/sesh", |_req, ctx| async move {
            let id = ctx.param("id").unwrap();
            let client = rankings::Client::new(&ctx, id)?;
            let template = include_str!("../content/session.html");
            let mut tt = TinyTemplate::new();
            tt.add_template("/session", template)
                .map_err(|err| err.to_string())?;

            let id = ctx.param("id").unwrap();

            let players: HashMap<u16, Player<RatingType>> =
                client.fetch("/players", "", Method::Get).await?;
            let session: Option<Session> = client.fetch("/session", "", Method::Get).await?;
            #[derive(Serialize)]
            struct PlayerString {
                name: String,
                id: u16,
            }

            #[derive(Serialize)]
            struct Context {
                id: String,
                session: bool,
                players: Vec<PlayerString>,
                session_players: Vec<PlayerString>,
            }

            let players_string = players
                .clone()
                .into_iter()
                .map(|(id, player)| PlayerString {
                    name: player.name,
                    id,
                })
                .collect();

            let session_players: Vec<PlayerString> = match session {
                Some(ref s) => s
                    .players
                    .keys()
                    .map(|id| PlayerString {
                        name: players.get(id).unwrap().name.clone(),
                        id: *id,
                    })
                    .collect(),
                None => vec![],
            };

            let context = Context {
                id: id.clone(),
                session: session.is_some(),
                players: players_string,
                session_players,
            };

            let mut rendered = tt
                .render("/session", &context)
                .map_err(|err| err.to_string())?;
            rendered.push_str(scripts::SESSION);
            Response::from_html(rendered)
        })
        .get_async("/:id", |_req, ctx| async move {
            let id = ctx.param("id").unwrap();
            let client = rankings::Client::new(&ctx, id)?;
            let template = include_str!("../content/index.html");
            let mut tt = TinyTemplate::new();
            tt.add_template("/", template)
                .map_err(|err| err.to_string())?;

            #[derive(Serialize)]
            struct MatchString {
                winners: Vec<String>,
                losers: Vec<String>,
            }

            #[derive(Serialize)]
            struct PlayerString {
                rank: usize,
                name: String,
                score: isize,
                wins: u16,
                losses: u16,
            }

            #[derive(Serialize)]
            struct Context {
                id: String,
                matches: Vec<MatchString>,
                players: Vec<PlayerString>,
            }

            let p = client.fetch("/players", "", Method::Get).await;

            let players: HashMap<u16, Player<RatingType>> = match p {
                Ok(players) => players,
                Err(e) => return Response::error(e.to_string(), 404),
            };
            let matches: Vec<Match> = client.fetch("/matches", "", Method::Get).await?;

            let matches_string = matches
                .iter()
                .rev()
                .take(15)
                .map(|m| MatchString {
                    winners: m
                        .team1
                        .iter()
                        .map(|winner| players.get(winner).unwrap().name.clone())
                        .collect(),
                    losers: m
                        .team2
                        .iter()
                        .map(|loser| players.get(loser).unwrap().name.clone())
                        .collect(),
                })
                .collect();

            let mut players_vec: Vec<&Player<RatingType>> = players.values().collect();
            players_vec
                .sort_by(|a, b| (b.rating.rating() as isize).cmp(&(a.rating.rating() as isize)));
            let players_string = players_vec
                .iter()
                .enumerate()
                .map(|(index, player)| PlayerString {
                    rank: index + 1,
                    name: player.name.clone(),
                    score: player.rating.rating() as isize,
                    wins: player.wins,
                    losses: player.wins,
                })
                .collect();

            let context = Context {
                id: id.clone(),
                matches: matches_string,
                players: players_string,
            };

            let mut rendered = tt.render("/", &context).map_err(|err| err.to_string())?;
            rendered.push_str(scripts::INDEX);
            Response::from_html(rendered)
        })
        .get_async("/", |_req, _ctx| async move {
            Response::from_html(include_str!("../content/create.html"))
        })
        .run(req, env)
        .await
}
