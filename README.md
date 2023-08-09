<div align="center">
  <h1><code>ubs-bot</code></h1>
  <p>
    <a href="https://github.com/ok-nick/ubs-bot/actions?query=workflow"><img src="https://github.com/ok-nick/ubs-bot/workflows/check/badge.svg" alt="check" /></a>
    <a href="https://docs.rs/ubs-bot"><img src="https://img.shields.io/readthedocs/ubs-bot" alt="docs" /></a>
    <a href="https://crates.io/crates/ubs-bot"><img src="https://img.shields.io/crates/v/ubs-bot" alt="crates" /></a>
    <a href="https://discord.gg/w9Bc6xH7uC"><img src="https://img.shields.io/discord/834969350061424660?label=discord" alt="discord" /></a>
  </p>
</div>

`ubs-bot` is a Discord bot for querying University at Buffalo class schedules in real-time and for receiving update notifications (such as class openings).

For more information, be sure to check out the [core `ubs` library](https://github.com/ok-nick/ubs).

## Installation
### Cargo
```bash
$ cargo install --git https://github.com/ok-nick/ubs-bot
```

## FAQ
### Why can't it find a class that I know exists?
`ubs-bot` is based off a predefined set of classes which, at the moment, does not span the entire course catalog. This is a fundamental issue, stemmed from the course to id mapping requirements by the backend network API. For more information, check out [this  issue](https://github.com/ok-nick/ubs/issues/1). If you would like to request a class, feel free to leave a comment [here](https://github.com/ok-nick/ubs/issues/1). If you are lazy, use the `raw` command counterparts to send raw ids to the bot.

### How often does the bot update its course information?
For course information queries, `ubs-bot` updates in real-time. However, for course watches, updates happen at a predefined interval set by the developer.