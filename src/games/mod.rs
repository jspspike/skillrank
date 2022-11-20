pub mod matchmaking;

use crate::rankings::{Client, Player, Match};

use std::collections::HashMap;

use worker::*;

async fn add_match(winners: Vec<u16>, losers: Vec<u16>, client: Client) -> Result<()> {
    let players: HashMap<u16, Player> = client.fetch("/players", None, Method::Get).await?;

    let winners_scores = winners.iter().map(|winner| { players.get(winner) });
    let losers_scores = losers.iter().map(|winner| { players.get(winner) });

    Ok(())
}
