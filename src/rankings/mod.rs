mod matches;
mod players;
mod session;
use matches::Match;
use players::{Player, PlayerCreate};

use serde::{Deserialize, Serialize};
use worker::*;

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
                players::setup(&self.state).await?;
                matches::setup(&self.state).await?;
                session::reset(&self.state).await?;

                Response::ok("")
            }
            "/players" => match req.method() {
                Method::Get => {
                    let players = players::get(&self.state).await?;
                    Response::ok(serde_json::to_string(&players)?)
                }
                Method::Post => {
                    let body: PlayerCreate = req.clone()?.json().await?;

                    players::create(&self.state, body).await?;
                    Response::ok("")
                }
                _ => Response::error("Not Found", 404),
            },
            "/matches" => match req.method() {
                Method::Get => {
                    let matches = matches::get(&self.state).await?;
                    Response::ok(serde_json::to_string(&matches)?)
                }
                Method::Post => {
                    let body: Match = req.clone()?.json().await?;

                    matches::create(&self.state, body).await?;
                    Response::ok("")
                }
                _ => Response::error("Not Found", 404),
            },
            "/session" => match req.method() {
                Method::Get => {
                    let session = session::get(&self.state).await?;
                    Response::ok(serde_json::to_string(&session)?)
                }
                Method::Put => {
                    let body: Vec<u16> = req.clone()?.json().await?;

                    session::start(&self.state, body).await?;
                    Response::ok("")
                }
                Method::Post => {
                    let body: Vec<u16> = req.clone()?.json().await?;

                    session::add_match(&self.state, body).await?;
                    Response::ok("")
                }
                Method::Delete => {
                    session::reset(&self.state).await?;
                    Response::ok("")
                }
                _ => Response::error("Not Found", 404),
            },
            _ => Response::error("Not Found", 404),
        }
    }
}
