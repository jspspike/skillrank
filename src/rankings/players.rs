use std::collections::HashMap;

use crate::RatingType;

use serde::{Deserialize, Serialize};
use worker::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerCreate {
    name: String,
    score: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Player {
    pub(crate) name: String,
    pub(crate) score: RatingType,
    pub(crate) wins: u16,
    pub(crate) losses: u16,
}

pub async fn setup(state: &State) -> Result<()> {
    state.storage().put("next_player_id", 0).await?;

    let players: HashMap<u16, Player> = HashMap::new();
    state.storage().put("players", players).await
}

pub async fn get(state: &State) -> Result<HashMap<u16, Player>> {
    state.storage().get("players").await
}

pub async fn create(state: &State, create: PlayerCreate) -> Result<()> {
    let next_player_id: u16 = state.storage().get("next_player_id").await?;
    let mut players: HashMap<u16, Player> = state.storage().get("players").await?;

    let score = match create.score {
        Some(rating) => RatingType {
            rating: rating as f64,
            uncertainty: 25.0 / 3.0,
        },
        None => RatingType::new(),
    };

    let new_player = Player {
        name: create.name,
        score,
        wins: 0,
        losses: 0,
    };
    players.insert(next_player_id, new_player);

    state.storage().put("players", players).await?;
    state
        .storage()
        .put("next_player_id", next_player_id + 1)
        .await
}
