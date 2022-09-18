use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use worker::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct Session {
    players: HashMap<u16, u16>,
}

pub async fn reset(state: &State) -> Result<()> {
    let session: Option<Session> = None;

    state.storage().put("session", session).await
}

pub async fn start(state: &State, players: Vec<u16>) -> Result<()> {
    let mut p: HashMap<u16, u16> = HashMap::new();

    players.iter().for_each(|player| {
        p.insert(*player, 0);
    });

    let session: Option<Session> = Some(Session { players: p });
    state.storage().put("session", session).await
}

pub async fn get(state: &State) -> Result<Option<Session>> {
    state.storage().get("session").await
}

pub async fn add_match(state: &State, players: Vec<u16>) -> Result<Session> {
    let session: Option<Session> = state.storage().get("session").await?;

    if let Some(mut session) = session {
        players.iter().for_each(|player| {
            session
                .players
                .entry(*player)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        });

        state.storage().put("session", &session).await?;
        return Ok(session);
    } else {
        return Err(Error::RouteNoDataError);
    }
}
