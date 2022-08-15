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
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use core::time::Duration;
use octocrab::{models, params};

async fn get_new_pull_requests(org: &str, repo: &str) -> Result<()> {
    let octocrab = octocrab::instance();
    // Returns the first page of all new prs.
    let mut page = octocrab
        .issues(org, repo)
        .list()
        .state(params::State::Open)
        .per_page(50)
        .send()
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_prs_should_work() {
        assert!(true);
    }
}
