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

use chrono::{DateTime, Utc};
use octocrab::models::{issues, pulls, IssueState};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PullRequest {
    pub pr_number: u64,
    pub title: Option<String>,
    pub body: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub merged_at: Option<DateTime<Utc>>,
    pub status: Option<IssueState>,
    pub url: Option<Url>,
}

impl From<pulls::PullRequest> for PullRequest {
    fn from(pr: pulls::PullRequest) -> Self {
        Self {
            pr_number: pr.number as u64,
            title: pr.title,
            body: pr.body,
            created_at: pr.created_at,
            updated_at: pr.updated_at,
            merged_at: pr.merged_at,
            status: pr.state,
            url: pr.html_url,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Issue {
    pub issue_number: u32,
    pub title: String,
    pub body: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub status: IssueState,
    pub url: Url,
}

impl From<issues::Issue> for Issue {
    fn from(issue: issues::Issue) -> Self {
        Self {
            issue_number: issue.number as u32,
            title: issue.title,
            body: issue.body,
            created_at: issue.created_at,
            updated_at: issue.updated_at,
            closed_at: issue.closed_at,
            status: IssueState::Open,
            url: issue.html_url,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub enum Status {
    #[default]
    Open,
    Closed,
    Merged,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct DbKey<'a> {
    pub repository: &'a str,
    pub pr_number: Option<u64>,
    pub issue_number: Option<u64>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Repository {
    pub organization: String,
    pub repository: String,
    pub query_release: bool,
}
