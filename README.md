# [skillrank.games](https://skillrank.games)

Website to track match results, get ratings for players, and perform matchmaking for your games. Inspired by [elovation](https://github.com/elovation/elovation) and built on [Cloudflare Workers](https://workers.cloudflare.com/). Here's an [example leaderboard](https://skillrank.games/example) to check out. I mostly made this website for myself meaning the frontend isn't very polished and a little brittle. Feel free to leave issues and PR's for any problems or features you'd like to see addressed.

This uses [trueskill](https://www.microsoft.com/en-us/research/project/trueskill-ranking-system/) to handle ratings but with some tweaking can easily use any rating system supported in the [skillratings crate](https://github.com/atomflunder/skillratings).
