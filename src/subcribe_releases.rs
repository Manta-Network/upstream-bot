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

use crate::types::DbKey;
use anyhow::Result;
use octocrab::models::repos;

// Get latest release.
pub async fn get_latest_release(org: &str, repo: &str) -> Result<repos::Release> {
    let latest_release = octocrab::instance()
        .repos(org, repo)
        .releases()
        .get_latest()
        .await?;

    Ok(latest_release)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_latest_release_should_work() {
        let (org, repo) = ("paritytech", "polkadot");
        assert!(get_latest_release(org, repo).await.is_ok());
    }
}
