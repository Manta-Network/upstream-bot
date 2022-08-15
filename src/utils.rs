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
use std::{
    fs::File,
    io::{prelude::*, BufReader},
};
use toml::Value;

// read project config file
pub fn blog_config() -> Result<Value> {
    let config = File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/config.toml"))?;
    let mut buff = BufReader::new(config);
    let mut contents = String::new();
    buff.read_to_string(&mut contents)?;

    let value = contents.parse::<Value>()?;
    Ok(value)
}

// configure sled db
pub fn db_config() -> sled::Result<sled::Db> {
    let db = sled::Config::default()
        // create a folder for store database file
        .path(concat!(env!("CARGO_MANIFEST_DIR"), "/db/"))
        .cache_capacity(1000_000_000) // size of databse file, 1Gb
        .flush_every_ms(Some(1000))
        .use_compression(true)
        .open();

    db
}
