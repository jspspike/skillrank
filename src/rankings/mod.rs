mod matches;
mod pass;
mod players;
mod session;

use crate::RatingType;
pub(crate) use matches::Match;
pub(crate) use players::{Player, PlayerCreate};
pub(crate) use session::{Session, SessionCreate};

use futures::try_join;
use std::collections::HashMap;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use worker::*;

pub struct Client {
    stub: Stub,
}

impl Client {
    pub fn new(ctx: &RouteContext<()>, name: &str) -> Result<Self> {
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
        let body = to_value(&string).ok().filter(|str| str != "\"\"");

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

    pub async fn check_pass(&self, req: &Request) -> Result<bool> {
        if req.method() == Method::Get {
            return Ok(true);
        }

        let pass = req.headers().get("passphrase").unwrap().unwrap();
        let ok = self.fetch("/pass", &pass, Method::Post).await?;
        Ok(ok)
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
        console_log!("{:?}", req);
        let salt = self.env.secret("PASS_SALT")?.to_string();

        match req.path().split('/').last().unwrap() {
            "pass" => match req.method() {
                Method::Get => {
                    let result = pass::get(&self.state).await?;
                    Response::from_json(&result)
                }
                Method::Post => {
                    let passphrase: String = req.clone()?.json().await?;
                    let ok = pass::check(&self.state, passphrase, salt).await?;
                    Response::from_json(&ok)
                }
                Method::Delete => {
                    pass::delete(&self.state).await?;
                    Response::from_json(&Empty {})
                }
                _ => Response::error("Not found", 404),
            },
            "setup" => {
                let pass: String = req.clone()?.json().await?;

                let players_fut = players::setup(&self.state);
                let matches_fut = matches::setup(&self.state);
                let session_fut = session::reset(&self.state);

                try_join!(players_fut, matches_fut, session_fut)?;
                pass::set(&self.state, pass, salt).await?;

                Response::from_json(&Empty {})
            }
            "players" => match req.method() {
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
            "matches" => match req.method() {
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
            "session" => match req.method() {
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

                    let session = session::add_match(&self.state, body).await?;
                    Response::from_json(&session)
                }
                Method::Patch => {
                    let body: Vec<u16> = req.clone()?.json().await?;

                    let session = session::add_player(&self.state, body).await?;
                    Response::from_json(&session)
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
