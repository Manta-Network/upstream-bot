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
use octocrab::models::{issues, pulls};
use sled::Db;
use std::sync::Arc;

pub async fn get_all_archived_issues(db: Arc<Db>, key_prefix: &[u8]) -> Result<Vec<issues::Issue>> {
    let mut iter = db.scan_prefix(key_prefix);
    let mut all_issues = vec![];
    while let Some(Ok((_key, val))) = iter.next() {
        let issue = serde_json::from_slice(val.as_ref())?;
        all_issues.push(issue);
    }
    Ok(all_issues)
}

pub async fn insert_one_issue(db: Arc<Db>, key_prefix: &str, issue: &issues::Issue) -> Result<()> {
    let key = format!("{key_prefix}#{0}", issue.number);
    let val = serde_json::to_vec(&issue)?;

    db.insert(key.as_bytes(), val)?;
    Ok(())
}

pub async fn insert_batch_issues(
    db: Arc<Db>,
    key_prefix: &str,
    new_issues: &[issues::Issue],
) -> Result<()> {
    let mut batch = sled::Batch::default();
    for issue in new_issues {
        let key = format!("{key_prefix}#{0}", issue.number);
        let val = serde_json::to_vec(&issue)?;
        batch.insert(key.as_bytes(), val);
    }

    db.apply_batch(batch)?;
    Ok(())
}

pub async fn get_all_archived_prs(
    db: Arc<Db>,
    key_prefix: &[u8],
) -> Result<Vec<pulls::PullRequest>> {
    let mut iter = db.scan_prefix(key_prefix);
    let mut all_prs = vec![];
    while let Some(Ok((_key, val))) = iter.next() {
        let pr = serde_json::from_slice(val.as_ref())?;
        all_prs.push(pr);
    }
    Ok(all_prs)
}

pub async fn insert_one_pr(db: Arc<Db>, key_prefix: &str, pr: &pulls::PullRequest) -> Result<()> {
    let key = format!("{key_prefix}#{0}", pr.number);
    let val = serde_json::to_vec(&pr)?;

    db.insert(key.as_bytes(), val)?;
    Ok(())
}

pub async fn insert_batch_prs(
    db: Arc<Db>,
    key_prefix: &str,
    new_prs: &[pulls::PullRequest],
) -> Result<()> {
    let mut batch = sled::Batch::default();
    for pr in new_prs {
        let key = format!("{key_prefix}#{0}", pr.number);
        let val = serde_json::to_vec(&pr)?;
        batch.insert(key.as_bytes(), val);
    }

    db.apply_batch(batch)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_all_archived_issues_should_work() {
        let db = crate::utils::db_config().unwrap();
        let octo = octocrab::instance();
        let (org, repo) = ("Manta-Network", "docs");
        let issues = crate::subcribe_issues::get_all_open_issues(octo.clone(), org, repo)
            .await
            .unwrap();
        for issue in issues.iter() {
            let key = format!("{org}#{repo}#issues#open#{0}", issue.number);
            let val = serde_json::to_vec(&issue).unwrap();
            db.insert(key.as_bytes(), val).unwrap();
        }

        let key_bytes = format!("{org}#{repo}#issues#");
        let all_issues = get_all_archived_issues(db.clone(), key_bytes.as_bytes())
            .await
            .unwrap();
        for issue in all_issues {
            dbg!(issue.html_url.to_string(), issue.title, issue.body);
        }

        let key_bytes = format!("{org}#{repo}#");
        let all_issues = get_all_archived_issues(db, key_bytes.as_bytes())
            .await
            .unwrap();
        for issue in all_issues {
            dbg!(issue.html_url.to_string(), issue.title, issue.body);
        }
    }
}
