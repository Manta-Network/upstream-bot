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
use chrono::prelude::*;
use octocrab::{models::pulls, params, Octocrab};
use sled::Db;
use std::sync::Arc;

pub type OpenPRs = Vec<pulls::PullRequest>;
pub type MergedPRs = Vec<pulls::PullRequest>;
pub type ClosedPRs = Vec<pulls::PullRequest>;

pub async fn get_all_merged_prs_by_date(
    octocrab: Arc<Octocrab>,
    org: &str,
    repo: &str,
    state: params::State,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Result<OpenPRs> {
    let mut page = octocrab
        .pulls(org, repo)
        .list()
        .state(state)
        .sort(params::pulls::Sort::Updated)
        .direction(params::Direction::Descending)
        .per_page(100)
        .send()
        .await?;

    let mut all_prs = vec![];
    let mut pull_times = 0u32;
    'query_pr: loop {
        for pr in &page {
            // seems there's a bug, some PRs will be returned.
            // so need to filter PRs.
            // not merged pr will be filtered.
            if pr
                .html_url
                .as_ref()
                .map(|url| url.as_str().contains("issues"))
                == Some(true)
                || pr.merged_at.is_none()
            {
                continue;
            }
            println!(
                "page: {pull_times}, prs: {}, pr title: {:?}, pr number: {}, date: {:?}",
                all_prs.len(),
                pr.html_url.as_ref().map(|s| s.to_string()),
                pr.number,
                pr.merged_at
            );

            // There're several situation(currently, there's no way to sort PRs by merged date in octocrab),
            // and updated_at >= merged_at always:
            // 1. both merged_at and updated_at in (from, to), record the pr.
            // 2. merged_at in (from, to), but updated_at in (t0, ...), record the pr.
            // 3. merged_at in (..., from), but updated_at in (from, to), continue.
            // 4. both merged_at and updated_at in (..., from), break the whole loop,
            //    because the current page of result is sorted by updated date.
            // 5. both merged_at and updated_at in (to, ...), continue, they are more newer pr.

            // situation 4
            if pr.merged_at < Some(from) && pr.updated_at < Some(from) {
                break 'query_pr;
            }
            // situation 3
            if pr.merged_at < Some(from) && pr.updated_at >= Some(from) && pr.updated_at <= Some(to)
            {
                continue;
            }
            // situation 5
            if pr.merged_at > Some(to) && pr.updated_at > Some(to) {
                continue;
            }
            // situation 1, 2
            if pr.merged_at >= Some(from) && pr.merged_at <= Some(to) {
                all_prs.push(pr.clone());
            }
        }
        page = match octocrab.get_page::<pulls::PullRequest>(&page.next).await? {
            Some(next_page) => next_page,
            None => break,
        };
        pull_times += 1;
    }

    Ok(all_prs)
}

pub async fn get_all_open_prs(
    octocrab: Arc<Octocrab>,
    org: &str,
    repo: &str,
    state: params::State,
) -> Result<OpenPRs> {
    let mut page = octocrab
        .pulls(org, repo)
        .list()
        .state(state)
        .per_page(50)
        .send()
        .await?;

    let mut all_prs = vec![];
    loop {
        for pr in &page {
            // seems there's a bug, some PRs will be returned.
            // so need to filter PRs.
            if pr
                .html_url
                .as_ref()
                .map(|url| url.as_str().contains("issues"))
                == Some(true)
                || pr.merged_at.is_none()
            {
                continue;
            }
            all_prs.push(pr.clone());
        }
        page = match octocrab.get_page::<pulls::PullRequest>(&page.next).await? {
            Some(next_page) => next_page,
            None => break,
        };
    }

    Ok(all_prs)
}

// return closed issues
pub async fn update_pr_status(
    db: Arc<Db>,
    org: &str,
    repo: &str,
) -> Result<(OpenPRs, MergedPRs, ClosedPRs)> {
    let octocrab = octocrab::instance();

    let open_prs = get_all_open_prs(octocrab.clone(), org, repo, params::State::Open).await?;
    // insert open prs
    let key_prefix = format!("{org}#{repo}#prs#open");
    crate::db::insert_batch_prs(db.clone(), &key_prefix, &open_prs).await?;

    let existing_prs = crate::db::get_all_archived_prs(db.clone(), key_prefix.as_bytes()).await?;

    // pr has 3 status: open, merged, closed
    let mut closed_prs = vec![];
    let mut merged_prs = vec![];
    for old_pr in existing_prs.iter() {
        // if old pr is not in current open prs, that means this pr has been closed or merged.
        if !open_prs.contains(old_pr) {
            // find out this pr is merged or closed.
            let pr = get_pr_by_id(octocrab.clone(), org, repo, old_pr.number).await?;
            // if pr is merged, merged_at is the concrete time.
            let old_key_prefix = format!("{org}#{repo}#prs#open#{0}", old_pr.number);
            let _ = db.remove(old_key_prefix.as_bytes())?;

            // 1. (Some(_), Some(_)) means the pr has been merged.
            // 2. (None, Some(_)) means the pr has been closed.
            // 3. (Some(_), None) seems impossible.
            // 4. (None, None) means it's a open pr.
            match (pr.merged_at, pr.closed_at) {
                (Some(_), Some(_)) => {
                    // delete the pr if it has been merged.
                    let new_key_prefix = format!("{org}#{repo}#prs#merged");
                    crate::db::insert_one_pr(db.clone(), &new_key_prefix, old_pr).await?;
                    merged_prs.push(old_pr.clone());
                }
                (None, Some(_)) => {
                    // delete the pr if it has been closed.
                    let new_key_prefix = format!("{org}#{repo}#prs#closed");
                    crate::db::insert_one_pr(db.clone(), &new_key_prefix, old_pr).await?;
                    closed_prs.push(old_pr.clone());
                }
                _ => (),
            }
        }
    }

    // find out new prs
    let mut new_prs = vec![];
    for open_pr in open_prs {
        if !existing_prs.contains(&open_pr) {
            new_prs.push(open_pr);
        }
    }

    Ok((new_prs, merged_prs, closed_prs))
}

pub async fn get_pr_by_id(
    octo: Arc<Octocrab>,
    org: &str,
    repo: &str,
    id: u64,
) -> Result<pulls::PullRequest> {
    let pr = octo.pulls(org, repo).get(id).await?;
    Ok(pr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use octocrab::params;
    use polars::prelude::*;

    #[tokio::test]
    async fn get_prs_should_work() {
        let db = crate::utils::db_config().unwrap();
        let octocrab = octocrab::instance();
        let (org, repo) = ("Manta-Network", "Manta");
        // 871 is a closed pr.
        let pr = get_pr_by_id(octocrab.clone(), org, repo, 871)
            .await
            .unwrap();
        assert!(pr.merged_at.is_none());
        assert!(pr.closed_at.is_some());

        // 867 is a merged pr.
        let pr = get_pr_by_id(octocrab.clone(), org, repo, 867)
            .await
            .unwrap();
        assert!(pr.merged_at.is_some());
        assert!(pr.closed_at.is_some());
    }

    #[tokio::test]
    async fn get_merged_prs_by_date_should_work() {
        let octocrab = octocrab::instance();
        let (org, repo) = ("paritytech", "substrate");
        let (from, to) = ("2022-11-24", "2022-11-26");
        let (from, to) = crate::utils::parse_from_date_and_to_date(from, to).unwrap();
        let merged_prs = get_all_merged_prs_by_date(
            octocrab.clone(),
            org,
            repo,
            params::State::Closed,
            from,
            to,
        )
        .await
        .unwrap();
        for pr in merged_prs {
            assert!(pr.merged_at.is_some());
        }
    }

    #[tokio::test]
    async fn format_prs_by_polars_should_work() {
        let octocrab = octocrab::instance();
        let (org, repo) = ("paritytech", "substrate");
        let (from, to) = ("2022-11-24", "2022-11-26");
        let (from, to) = crate::utils::parse_from_date_and_to_date(from, to).unwrap();
        let merged_prs = get_all_merged_prs_by_date(
            octocrab.clone(),
            org,
            repo,
            params::State::Closed,
            from,
            to,
        )
        .await
        .unwrap();
        let mut link_list = Vec::with_capacity(merged_prs.len());
        let mut title_list = Vec::with_capacity(merged_prs.len());
        for pr in merged_prs {
            assert!(pr.merged_at.is_some());
            link_list.push(pr.html_url.map(|s| s.to_string()));
            title_list.push(pr.title.map(|s| format!("**{0}**", s.trim())));
        }
        let mut df = df! [
            "title" => title_list,
            "link"  => link_list,
        ]
        .unwrap();
        println!("{df}");
        let mut file = std::fs::File::create("prs.csv").unwrap();
        CsvWriter::new(&mut file).finish(&mut df).unwrap();
    }
}
