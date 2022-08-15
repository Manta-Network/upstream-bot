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

use crate::types::{DbKey, PullRequest};
use anyhow::Result;
use octocrab::{models, params};
use sled::Db;
use std::sync::Arc;

// Get new PRs.
async fn get_new_pull_requests(db: Arc<Db>, org: &str, repo: &str) -> Result<Vec<PullRequest>> {
    let octocrab = octocrab::instance();
    let mut page = octocrab
        .pulls(org, repo)
        .list()
        .state(params::State::Open)
        .per_page(50)
        .send()
        .await?;

    let mut new_prs: Vec<PullRequest> = vec![];
    loop {
        for pr in &page {
            let new_pr = PullRequest::from(pr.clone());
            // seems there's a bug, some issues will be returned.
            // so need to filter issues.
            if let Some(true) = pr
                .html_url
                .as_ref()
                .map(|url| url.as_str().contains("issues"))
            {
                continue;
            }

            let key = DbKey {
                repository: repo,
                pr_number: Some(pr.number as u64),
                issue_number: None,
            };
            let key_bytes = bincode::serialize(&key)?;
            if db.contains_key(&key_bytes)? {
                println!("old issue will not be inserted to db: {}", new_pr.pr_number);
            } else {
                let val_bytes = bincode::serialize(&new_pr)?;
                db.insert(key_bytes, val_bytes)?;
                println!(
                    "new issue inserted: {:?}, url: {:?}",
                    new_pr.pr_number,
                    new_pr.url.as_ref().map(|url| url.as_str())
                );
                new_prs.push(new_pr);
            }
        }
        page = match octocrab
            .get_page::<models::pulls::PullRequest>(&page.next)
            .await?
        {
            Some(next_page) => next_page,
            None => break,
        }
    }

    Ok(new_prs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn get_prs_should_work() {
        let db = crate::utils::db_config().unwrap();
        let db = Arc::new(db);
        let (org, repo) = ("open-web3-stack", "open-runtime-module-library");
        let _ = get_new_pull_requests(db.clone(), org, repo).await;
        assert!(true);

        let key = DbKey {
            repository: "open-runtime-module-library",
            pr_number: Some(379),
            issue_number: None,
        };
        let bytes_key = bincode::serialize(&key).unwrap();
        let val = db.get(&bytes_key).unwrap().unwrap();
        let v: crate::types::PullRequest = bincode::deserialize(&val).unwrap();
        dbg!(v);
    }
}
