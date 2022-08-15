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

use crate::types::{DbKey, Issue};
use anyhow::Result;
use octocrab::{models, params};
use sled::Db;
use std::sync::Arc;

// Get new issues.
async fn get_new_issues(db: Arc<Db>, org: &str, repo: &str) -> Result<Vec<Issue>> {
    let octocrab = octocrab::instance();
    let mut page = octocrab
        .issues(org, repo)
        .list()
        .state(params::State::Open)
        .per_page(50)
        .send()
        .await?;

    let mut new_issues: Vec<Issue> = vec![];
    loop {
        for issue in &page {
            let _issue = Issue::from(issue.clone());
            // seems there's a bug, some PRs will be returned.
            // so need to filter PRs.
            if issue.html_url.as_str().contains("pull") {
                continue;
            }

            let key = DbKey {
                repository: repo,
                pr_number: None,
                issue_number: Some(issue.number as u64),
            };
            let key_bytes = bincode::serialize(&key)?;
            if db.contains_key(&key_bytes)? {
                println!(
                    "old issue will not be inserted to db: {}",
                    _issue.issue_number
                );
            } else {
                let val_bytes = bincode::serialize(&_issue)?;
                db.insert(key_bytes, val_bytes)?;
                println!(
                    "new issue inserted: {:?}, url: {}",
                    _issue.issue_number,
                    _issue.url.as_str()
                );
                new_issues.push(_issue);
            }
        }
        page = match octocrab
            .get_page::<models::issues::Issue>(&page.next)
            .await?
        {
            Some(next_page) => next_page,
            None => break,
        }
    }

    Ok(new_issues)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn get_issues_should_work() {
        let db = crate::utils::db_config().unwrap();
        let db = Arc::new(db);
        let (org, repo) = ("open-web3-stack", "open-runtime-module-library");
        let _ = get_new_issues(db.clone(), org, repo).await;
        assert!(true);

        let key = DbKey {
            repository: "open-runtime-module-library",
            pr_number: None,
            issue_number: Some(118),
        };
        let bytes_key = bincode::serialize(&key).unwrap();
        let val = db.get(&bytes_key).unwrap().unwrap();
        let v: crate::types::Issue = bincode::deserialize(&val).unwrap();
        dbg!(v);
    }
}
