use std::collections::HashMap;

use worker::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Match {
    id: u16,
    team1: Vec<Player>,
    team2: Vec<Player>,
    team1_win: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Session {
    enabled: bool,
    active: Vec<Player>,
    nets: u8,
}

/// Durable Object storage for match and player data
#[durable_object]
pub struct Rankings {
    state: State,
    env: Env,
}

#[durable_object]
impl DurableObject for Rankings {
    fn new(state: State, env: Env) -> Self {
        Self { state, env }
    }

    async fn fetch(&mut self, req: Request) -> Result<Response> {
        match req.path().as_str() {
            "/setup" => {
                player::setup_player(&self.State);
                self.state.storage().put("next_match_id", 0).await?;

                // Maybe should use IndexMap at some point
                let players: HashMap<u16, Player> = HashMap::new();
                self.state.storage().put("players", players).await?;

                let matches: Vec<Match> = vec![];
                self.state.storage().put("matches", matches).await?;

                Response::ok("")
            }
            "/players" => match req.method() {
                Method::Get => {
                    let players: HashMap<u16, Player> = self.state.storage().get("players").await?;
                    Response::ok(serde_json::to_string(&players)?)
                }
                Method::Post => {
                    let next_player_id: u16 = self.state.storage().get("next_player_id").await?;
                    let mut players: HashMap<u16, Player> = self.state.storage().get("players").await?;

                    let body: PlayerCreate = req.clone()?.json().await?;

                    let new_player = Player {
                        name: body.name,
                        score: body.score,
                        wins: 0,
                        losses: 0,
                    };
                    players.insert(next_player_id, new_player);

                    self.state.storage().put("players", players).await?;
                    self.state
                        .storage()
                        .put("next_player_id", next_player_id + 1)
                        .await?;
                    Response::ok("")
                }
                _ => Response::error("Not Found", 404),
            },
            "/matches" => match req.method() {
                Method::Get => {
                    let matches: Vec<Player> = self.state.storage().get("matches").await?;
                    Response::ok(serde_json::to_string(&matches)?)
                }
                Method::Post => {
                    let next_match_id: u16 = self.state.storage().get("next_match_id").await?;
                    let mut matches: Vec<Match> = self.state.storage().get("matches").await?;

                    let body: Match = req.clone()?.json().await?;

                    let new_match = Match {
                        id: next_match_id,
                        team1: body.team1,
                        team2: body.team2,
                        team1_win: body.team1_win,
                    };
                    matches.push(new_match);

                    self.state.storage().put("matches", matches).await?;
                    self.state
                        .storage()
                        .put("next_player_id", next_match_id + 1)
                        .await?;
                    Response::ok("")
                }
                _ => Response::error("Not Found", 404),
            },
            _ => Response::error("Not Found", 404),
        }
    }
}
