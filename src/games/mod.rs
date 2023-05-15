pub mod matchmaking;

use crate::rankings::{Client, Empty, Match, Player};

use std::collections::HashMap;

use skillratings::{Outcomes, TeamRatingSystem};
use worker::*;

pub async fn add_match<RS: TeamRatingSystem>(
    winners: &[u16],
    losers: &[u16],
    client: Client,
    rating_system: RS,
) -> Result<()> {
    let mut players: HashMap<u16, Player<RS::RATING>> =
        client.fetch("/players", "", Method::Get).await?;

    let winners_ratings: Vec<RS::RATING> = winners
        .iter()
        .map(|winner| players.get(winner).unwrap().rating)
        .collect();
    let losers_ratings: Vec<RS::RATING> = losers
        .iter()
        .map(|loser| players.get(loser).unwrap().rating)
        .collect();

    console_log!("{:?}", winners_ratings);
    console_log!("{:?}", losers_ratings);
    let (winners_final, losers_final) =
        rating_system.rating(&winners_ratings, &losers_ratings, &Outcomes::WIN);
    console_log!("{:?}", winners_final);
    console_log!("{:?}", losers_final);

    winners
        .iter()
        .zip(winners_final.iter())
        .for_each(|(id, rating)| {
            let player = players.get_mut(id).unwrap();
            player.rating = *rating;
            player.wins += 1;
        });
    losers
        .iter()
        .zip(losers_final.iter())
        .for_each(|(id, rating)| {
            let player = players.get_mut(id).unwrap();
            player.rating = *rating;
            player.losses += 1;
        });

    let participants: Vec<u16> = [winners, losers].concat();
    let _: Empty = client
        .fetch("/session", &participants, Method::Post)
        .await?;

    let _: Empty = client.fetch("/players", &players, Method::Put).await?;

    let m = Match {
        id: 0,
        team1: winners.to_vec(),
        team2: losers.to_vec(),
    };
    let _: Empty = client.fetch("/matches", &m, Method::Post).await?;

    Ok(())
}
