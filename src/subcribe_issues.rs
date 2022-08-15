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
use octocrab::{models, params};

async fn get_new_issues(org: &str, repo: &str) -> Result<()> {
    let octocrab = octocrab::instance();
    // Returns the first page of all issues.
    let mut page = octocrab
        .issues(org, repo)
        .list()
        .state(params::State::Open)
        .per_page(50)
        .send()
        .await?;

    let mut a = 0u32;
    loop {
        for issue in &page {
            println!(
                "{}, {}, {}",
                // issue.title, issue.html_url, issue.body, issue.created_at
                issue.title,
                issue.html_url,
                issue.created_at
            );
            a += 1;
            // break;
        }
        page = match octocrab
            .get_page::<models::issues::Issue>(&page.next)
            .await?
        {
            Some(next_page) => next_page,
            None => break,
        }
    }
    dbg!(a);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_issues_should_work() {
        let (org, repo) = ("open-web3-stack", "open-runtime-module-library");
        get_new_issues(org, repo).await;
        assert!(true);
    }
}
