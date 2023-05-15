use serde::{Deserialize, Serialize};
use worker::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct Match {
    pub(crate) id: u16,
    pub(crate) team1: Vec<u16>, // Always the winning team
    pub(crate) team2: Vec<u16>,
}

pub async fn setup(state: &State) -> Result<()> {
    state.storage().put("next_match_id", 0).await?;

    let matches: Vec<Match> = vec![];
    state.storage().put("matches", matches).await
}

pub async fn get(state: &State) -> Result<Vec<Match>> {
    state.storage().get("matches").await
}

pub async fn create(state: &State, m: Match) -> Result<()> {
    let next_match_id: u16 = state.storage().get("next_match_id").await?;
    let mut matches: Vec<Match> = state.storage().get("matches").await?;

    let new_match = Match {
        id: next_match_id,
        team1: m.team1,
        team2: m.team2,
    };
    matches.push(new_match);

    state.storage().put("matches", matches).await?;
    state
        .storage()
        .put("next_match_id", next_match_id + 1)
        .await
}
