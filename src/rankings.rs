use worker::*;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct Player {
    id: u16,
    name: String,
    score: u16,
    wins: u16,
    losses: u16,
}

#[derive(Serialize, Deserialize, Debug)]
struct Match {
    team1: Vec<Player>,
    team2: Vec<Player>,
    team1_win: bool,
}

#[durable_object]
pub struct Rankings {
    state: State,
    env: Env,
}

#[durable_object]
impl DurableObject for Rankings {
    fn new(state: State, env: Env) -> Self {
        Self {
            state,
            env,
        }
    }

    async fn fetch(&mut self, req: Request) -> Result<Response> {
        match req.path().as_str() {
            "/" => {
                let players: Vec<Player> = self.state.storage().get("players").await?;
                Response::ok(format!("{:?}", players))
            },
            "/setup" => {
                self.state.storage().put("next_player_id", 0).await?;

                let players: Vec<Player> = vec![];
                self.state.storage().put("players", players).await?;

                let matches: Vec<Match> = vec![];
                self.state.storage().put("matches", matches).await?;
                Response::ok("")

            }
            "/add" => {
                let next_player_id: u16 = self.state.storage().get("next_player_id").await?;
                let mut players: Vec<Player> = self.state.storage().get("players").await?;

                let new_player = Player {
                    id: next_player_id,
                    name: "Bobby G".to_string(),
                    score: 0,
                    wins: 0,
                    losses: 0,
                };
                players.push(new_player);

                self.state.storage().put("players", players).await?;
                self.state.storage().put("next_player_id", next_player_id + 1).await?;
                Response::ok("")
            }
            _ => {
                Response::error("Not Found", 404)
            }
        }
    }
}
