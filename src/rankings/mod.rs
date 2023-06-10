mod matches;
mod players;
mod session;

use crate::RatingType;
pub(crate) use matches::Match;
pub(crate) use players::{Player, PlayerCreate};
pub(crate) use session::{Session, SessionCreate};

use std::collections::HashMap;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use worker::*;

pub struct Client {
    stub: Stub,
}

impl Client {
    pub fn new(ctx: RouteContext<()>, name: &str) -> Result<Self> {
        let namespace = ctx.durable_object("RANKINGS")?;
        let stub = namespace.id_from_name(name)?.get_stub()?;

        Ok(Client { stub })
    }

    pub async fn fetch<B: DeserializeOwned, T: ?Sized + Serialize>(
        &self,
        path: &str,
        value: &T,
        method: Method,
    ) -> Result<B> {
        let string = serde_json::to_string(&value)?;
        let body = match to_value(&string).ok() {
            Some(str) => {
                if str == "\"\"" {
                    None
                } else {
                    Some(str)
                }
            }
            None => None,
        };

        let req = Request::new_with_init(
            format!("https://w{}", path).as_str(),
            &RequestInit {
                body,
                headers: Headers::new(),
                cf: CfProperties::default(),
                method,
                redirect: RequestRedirect::Follow,
            },
        )?;

        self.stub.fetch_with_request(req).await?.json().await
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Empty {}

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

                Response::from_json(&Empty {})
            }
            "/players" => match req.method() {
                Method::Get => {
                    let players = players::get(&self.state).await?;
                    Response::from_json(&players)
                }
                Method::Post => {
                    let body: PlayerCreate = req.clone()?.json().await?;

                    players::create(&self.state, body).await?;
                    Response::from_json(&Empty {})
                }
                Method::Put => {
                    let body: HashMap<u16, Player<RatingType>> = req.clone()?.json().await?;
                    players::set(&self.state, body).await?;
                    Response::from_json(&Empty {})
                }
                _ => Response::error("Not Found", 404),
            },
            "/matches" => match req.method() {
                Method::Get => {
                    let matches = matches::get(&self.state).await?;
                    Response::from_json(&matches)
                }
                Method::Post => {
                    let body: Match = req.clone()?.json().await?;

                    matches::create(&self.state, body).await?;
                    Response::from_json(&Empty {})
                }
                _ => Response::error("Not Found", 404),
            },
            "/session" => match req.method() {
                Method::Get => {
                    let session = session::get(&self.state).await?;
                    Response::from_json(&session)
                }
                Method::Put => {
                    let body: SessionCreate = req.clone()?.json().await?;

                    session::start(&self.state, body).await?;
                    Response::from_json(&Empty {})
                }
                Method::Post => {
                    let body: Vec<u16> = req.clone()?.json().await?;

                    session::add_match(&self.state, body).await?;
                    Response::from_json(&Empty {})
                }
                Method::Delete => {
                    session::reset(&self.state).await?;
                    Response::from_json(&Empty {})
                }
                _ => Response::error("Not Found", 404),
            },
            _ => Response::error("Not Found", 404),
        }
    }
}
