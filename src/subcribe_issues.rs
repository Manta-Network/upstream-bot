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

//! Only query open issues

use anyhow::Result;
use chrono::prelude::*;
use octocrab::{models::issues, params, Octocrab};
use sled::Db;
use std::sync::Arc;

pub async fn get_open_issues_by_date(
    octocrab: Arc<Octocrab>,
    org: &str,
    repo: &str,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Result<Vec<issues::Issue>> {
    let mut page = octocrab
        .issues(org, repo)
        .list()
        .state(params::State::Open)
        .sort(params::issues::Sort::Created)
        .direction(params::Direction::Descending)
        .per_page(100)
        .send()
        .await?;

    let mut all_issues = vec![];
    'query_issue: loop {
        for issue in &page {
            // seems there's a bug, some PRs will be returned.
            // so need to filter PRs.
            if issue.html_url.as_str().contains("pull") {
                continue;
            }

            if issue.created_at > to {
                continue;
            }
            if issue.created_at >= from && issue.created_at <= to {
                all_issues.push(issue.clone());
            }
            if issue.created_at < from {
                break 'query_issue;
            }
        }

        // go to next page of issues.
        page = match octocrab.get_page::<issues::Issue>(&page.next).await? {
            Some(next_page) => next_page,
            None => break,
        }
    }

    Ok(all_issues)
}

pub async fn get_all_open_issues(
    octocrab: Arc<Octocrab>,
    org: &str,
    repo: &str,
) -> Result<Vec<issues::Issue>> {
    let mut page = octocrab
        .issues(org, repo)
        .list()
        .state(params::State::Open)
        .per_page(50)
        .send()
        .await?;

    let mut all_issues = vec![];
    loop {
        for issue in &page {
            // seems there's a bug, some PRs will be returned.
            // so need to filter PRs.
            if issue.html_url.as_str().contains("pull") {
                continue;
            }
            all_issues.push(issue.clone());
        }

        // go to next page of issues.
        page = match octocrab.get_page::<issues::Issue>(&page.next).await? {
            Some(next_page) => next_page,
            None => break,
        }
    }

    Ok(all_issues)
}

// return closed issues
pub async fn update_issue_status(db: Arc<Db>, org: &str, repo: &str) -> Result<Vec<issues::Issue>> {
    let octocrab = octocrab::instance();

    // pr has 2 status: open, closed
    let open_issues = get_all_open_issues(octocrab, org, repo).await?;
    // insert open issues
    let key_prefix = format!("{org}#{repo}#issues#open");
    crate::db::insert_batch_issues(db.clone(), &key_prefix, &open_issues).await?;

    let existing_issues =
        crate::db::get_all_archived_issues(db.clone(), key_prefix.as_bytes()).await?;

    let mut closed_issues = vec![];
    for old_issue in existing_issues {
        // if old issue is not in current open issues, that means this issue has been closed.
        if !open_issues.contains(&old_issue) {
            // delete the issue if it has been closed.
            let old_key_prefix = format!("{org}#{repo}#issues#open#{0}", old_issue.number);
            let _ = db.remove(old_key_prefix.as_bytes())?;
            let new_key_prefix = format!("{org}#{repo}#issues#closed");
            crate::db::insert_one_issue(db.clone(), &new_key_prefix, &old_issue).await?;
            closed_issues.push(old_issue);
        }
    }

    Ok(closed_issues)
}

pub async fn get_issue_by_id(
    octo: &Octocrab,
    id: u64,
    org: &str,
    repo: &str,
) -> Result<issues::Issue> {
    let issue = octo.issues(org, repo).get(id).await?;
    Ok(issue)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn get_issues_should_work() {
        let db = crate::utils::db_config().unwrap();
    }
}
