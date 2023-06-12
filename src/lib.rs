mod games;
mod rankings;
mod scripts;
mod utils;

use rankings::{Match, Player, Session};
use games::matchmaking;

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
        .on_async("/generate-matches", |mut req, ctx| async move {
            let client = rankings::Client::new(ctx, "Spikeball")?;
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

                format!("{}Game {}\nTeam 1:{}\nTeam 2:{}\n\n\n", acc,  m.id + 1, team_1, team_2)
            });
            Response::ok(matches_str)
        })
        .on_async("/add-match", |mut req, ctx| async move {
            let client = rankings::Client::new(ctx, "Spikeball")?;
            let m: Match = req.json().await?;
            let rating_system = TrueSkill::new(TrueSkillConfig {
                draw_probability: 0.0,
                beta: 100.0,
                default_dynamics: 0.15,
            });

            games::add_match(&m.team1, &m.team2, client, rating_system).await?;
            Response::ok("")
        })
        .get_async("/make", |req, ctx| async move {
            let client = rankings::Client::new(ctx, "Spikeball")?;

            let players: HashMap<u16, Player<RatingType>> =
                client.fetch("/players", "", Method::Get).await?;
            let session: Session = client.fetch("/session", "", Method::Get).await?;

            let curr = players.keys().copied().collect();

            let game_info = games::matchmaking::GameInfo {
                games: 2,
                players_per_team: 2,
                stability: 2.0,
            };

            let matches =
                games::matchmaking::generate_matches(curr, &players, session, game_info)?;
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
        .get_async("/sesh", |_req, ctx| async move {
            let client = rankings::Client::new(ctx, "Spikeball")?;
            let template = include_str!("../content/session.html");
            let mut tt = TinyTemplate::new();
            tt.add_template("/session", template)
                .map_err(|err| err.to_string())?;

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
                title: String,
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
                Some(ref s) => { s.players.keys().map(|id| {
                    PlayerString {
                        name: players.get(id).unwrap().name.clone(),
                        id: *id
                    }
                }).collect() },
                None => vec![],
            };

            let context = Context {
                title: "Spikeball".to_string(),
                session: session.is_some(),
                players: players_string,
                session_players
            };

            let mut rendered = tt
                .render("/session", &context)
                .map_err(|err| err.to_string())?;
            rendered.push_str(scripts::SESSION);
            Response::from_html(rendered)
        })
        .get_async("/", |_req, ctx| async move {
            let client = rankings::Client::new(ctx, "Spikeball")?;
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
            }

            #[derive(Serialize)]
            struct Context {
                title: String,
                matches: Vec<MatchString>,
                players: Vec<PlayerString>,
            }

            let players: HashMap<u16, Player<RatingType>> =
                client.fetch("/players", "", Method::Get).await?;
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
                })
                .collect();

            let context = Context {
                title: "Spikeball".to_string(),
                matches: matches_string,
                players: players_string,
            };

            let rendered = tt.render("/", &context).map_err(|err| err.to_string())?;
            Response::from_html(rendered)
        })
        .run(req, env)
        .await
}
