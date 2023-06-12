use std::collections::HashMap;

use crate::games::matchmaking::GameInfo;

use serde::{Deserialize, Serialize};
use worker::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Session {
    pub(crate) players: HashMap<u16, u16>,
    pub(crate) most_played: u16,
    pub(crate) game_info: GameInfo,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionCreate {
    pub(crate) players: Vec<u16>,
    pub(crate) game_info: GameInfo,
}

pub async fn reset(state: &State) -> Result<()> {
    let session: Option<Session> = None;

    state.storage().put("session", session).await
}

pub async fn start(state: &State, body: SessionCreate) -> Result<()> {
    let mut p: HashMap<u16, u16> = HashMap::new();

    body.players.iter().for_each(|player| {
        p.insert(*player, 0);
    });

    let session: Option<Session> = Some(Session {
        players: p,
        most_played: 0,
        game_info: body.game_info,
    });
    state.storage().put("session", session).await
}

pub async fn get(state: &State) -> Result<Option<Session>> {
    state.storage().get("session").await
}

pub async fn add_match(state: &State, players: Vec<u16>) -> Result<Session> {
    let session: Option<Session> = state.storage().get("session").await?;

    if let Some(mut session) = session {
        let mut most_played = session.most_played;
        players.iter().for_each(|player| {
            session
                .players
                .entry(*player)
                .and_modify(|count| {
                    *count += 1;
                    if *count > most_played {
                        most_played = *count
                    }
                })
                .or_insert(1);
        });
        session.most_played = most_played;

        state.storage().put("session", &session).await?;
        Ok(session)
    } else {
        Err(Error::RouteNoDataError)
    }
}

pub async fn add_player(state: &State, players: Vec<u16>) -> Result<Session> {
    let session: Option<Session> = state.storage().get("session").await?;
    if let Some(mut session) = session {
        players.iter().for_each(|player| {
            session.players.insert(*player, 0);
        });

        state.storage().put("session", &session).await?;
        Ok(session)
    } else {
        Err(Error::RouteNoDataError)
    }
}
