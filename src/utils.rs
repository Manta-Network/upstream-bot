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

use anyhow::Result;
use chrono::naive::Days;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{prelude::*, BufReader},
    sync::Arc,
};
use thiserror::Error;
use toml::Value;

#[derive(Error, Debug)]
pub enum IntenalError {
    #[error("Failed to parse toml file.")]
    TomlParseError,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Repository {
    pub organization: String,
    pub repository: String,
    pub query_release: bool,
}

// read project config file
pub fn read_config() -> Result<Value> {
    let config = File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/config.toml"))?;
    let mut buff = BufReader::new(config);
    let mut contents = String::new();
    buff.read_to_string(&mut contents)?;

    let value = contents.parse::<Value>()?;
    Ok(value)
}

pub fn get_repositories(config: &Value) -> Result<Vec<Repository>> {
    let mut repositories = vec![];
    for (k, _v) in config
        .as_table()
        .ok_or(IntenalError::TomlParseError)?
        .into_iter()
    {
        for (m, n) in config[k].as_table().as_ref().unwrap().into_iter() {
            if let Some(query_release) = n.get("query-release") {
                let repo = Repository {
                    organization: k.to_owned(),
                    repository: m.to_owned(),
                    query_release: query_release
                        .as_bool()
                        .ok_or(IntenalError::TomlParseError)?,
                };
                repositories.push(repo);
            }
        }
    }

    Ok(repositories)
}

pub fn get_update_frequence(config: &Value) -> i64 {
    config["schedule"]["frequence"]
        .as_integer()
        .expect("Please give a number less than 24h here.")
}

pub fn get_discord_token(config: &Value) -> &str {
    config["discord"]["bot-token"]
        .as_str()
        .expect("Please give discord token here.")
}

// configure sled db
pub fn db_config() -> sled::Result<Arc<sled::Db>> {
    sled::Config::default()
        // create a folder for store database file
        .path(concat!(env!("CARGO_MANIFEST_DIR"), "/db/"))
        .cache_capacity(1_000_000_000) // size of databse file, 1Gb
        .flush_every_ms(Some(1000))
        .open()
        .map(Arc::new)
}

pub fn parse_from_date_and_to_date(from: &str, to: &str) -> Result<(DateTime<Utc>, DateTime<Utc>)> {
    let to = format!("{to} 00:00:00");
    let to = {
        let _to = NaiveDateTime::parse_from_str(&to, "%Y-%m-%d %H:%M:%S")?;
        let to = DateTime::from_utc(_to + Days::new(1), Utc); // if to = 2022-11-25, actually which means 2022-11-26 00:00:00
        let now = Utc::now();
        if to > now {
            now
        } else {
            to
        }
    };

    let from = format!("{from} 00:00:00");
    let from = NaiveDateTime::parse_from_str(&from, "%Y-%m-%d %H:%M:%S")?;

    Ok((DateTime::from_utc(from, Utc), to))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn read_config_should_work() {
        let config = read_config().unwrap();
        let repos = get_repositories(&config).unwrap();
        assert_eq!(repos.len(), 6);
        assert_eq!(get_discord_token(&config), "123456789");
        assert_eq!(get_update_frequence(&config), 7200);
    }

    #[tokio::test]
    async fn ensure_every_repository_is_valid() {
        let config = read_config().unwrap();
        let all_repos = get_repositories(&config).unwrap();

        let octocrab = octocrab::instance();
        for repo in all_repos {
            assert!(octocrab
                .repos(&repo.organization, &repo.repository)
                .get()
                .await
                .is_ok());
        }
    }

    #[test]
    fn parse_date_should_work() {
        let (from, to) = ("2022-11-24", "2022-11-25");

        let _from = NaiveDateTime::parse_from_str(&format!("{from} 00:00:00"), "%Y-%m-%d %H:%M:%S")
            .unwrap();
        let _to =
            NaiveDateTime::parse_from_str(&format!("{to} 00:00:00"), "%Y-%m-%d %H:%M:%S").unwrap();
        let (from, to) = parse_from_date_and_to_date(from, to).unwrap();
        assert_eq!(from, DateTime::<Utc>::from_utc(_from, Utc));
        assert_eq!(to - DateTime::<Utc>::from_utc(_to, Utc), Duration::days(1));
    }
}
