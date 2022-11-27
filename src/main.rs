// Copyright 2020-2022 Manta Network.
// This file is part of Manta.
//
// Manta is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Manta is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Manta.  If not, see <http://www.gnu.org/licenses/>.

#![allow(dead_code)]

use anyhow::Result;
use clap::Parser;

mod cli;
mod db;
// mod discord_bot;
mod subcribe_issues;
mod subcribe_prs;
mod subcribe_releases;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = cli::DiscordBotCli::parse();
    let config = utils::read_config()?;

    match cli.get {
        Some(cli::Commands::Issue(args)) => {
            crate::cli::generate_issue_csv_report(&args).await?;
        }
        Some(cli::Commands::Pr(args)) => {
            crate::cli::generate_pr_csv_report(&args).await?;
        }
        None => (),
    }

    // discord_bot::discord_bot(&config).await;

    Ok(())
}
