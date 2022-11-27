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
use clap::{Args, Parser, Subcommand, ValueEnum};
use octocrab::params;
use polars::prelude::*;
use std::fs::{create_dir_all, File};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct DiscordBotCli {
    #[command(subcommand)]
    pub get: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Issue(Arguments),
    Pr(Arguments),
}

#[derive(Args, Debug)]
pub struct Arguments {
    #[arg(long, require_equals = true, ignore_case = true)]
    pub org: String,
    #[arg(long, require_equals = true, ignore_case = true)]
    pub repo: String,
    #[arg(long, require_equals = true, ignore_case = true)]
    pub from: String,
    #[arg(long, require_equals = true, ignore_case = true)]
    pub to: String,
    #[arg(long, require_equals = true, ignore_case = true, num_args = 0..=1, default_value_t = Status::Merged, value_enum)]
    pub status: Status,
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum Status {
    Open,
    Merged,
    Closed,
}

pub async fn generate_pr_csv_report(
    Arguments {
        org,
        repo,
        from,
        to,
        status,
    }: &Arguments,
) -> Result<()> {
    let _path = format!("./{repo}/{from} => {to}");

    let status = match status {
        Status::Open => params::State::Open,
        Status::Merged | Status::Closed => params::State::Closed,
    };

    let octocrab = octocrab::instance();
    let (from, to) = crate::utils::parse_from_date_and_to_date(from, to)?;
    let merged_prs =
        crate::subcribe_prs::get_all_merged_prs_by_date(octocrab, org, repo, status, from, to)
            .await?;
    let mut link_list = Vec::with_capacity(merged_prs.len());
    let mut title_list = Vec::with_capacity(merged_prs.len());
    let mut merged_date_list = Vec::with_capacity(merged_prs.len());
    for pr in merged_prs {
        link_list.push(pr.html_url.map(|s| s.to_string()));
        title_list.push(pr.title.map(|s| format!("**{0}**", s.trim())));
        merged_date_list.push(pr.merged_at.map(|d| d.to_string()));
    }
    let mut df = df![
        "merged date"  => merged_date_list,
        "title" => title_list,
        "link"  => link_list,
    ]?;

    println!("{repo}'s prs report: {df}");
    create_dir_all(&_path)?;

    let csv_path = format!("{_path}/pr.csv");
    println!("The report has been generated at: {:?}.", csv_path);
    let mut file = File::create(csv_path)?;
    CsvWriter::new(&mut file).finish(&mut df)?;

    Ok(())
}

pub async fn generate_issue_csv_report(
    Arguments {
        org,
        repo,
        from,
        to,
        ..
    }: &Arguments,
) -> Result<()> {
    let _path = format!("./{repo}/{from} => {to}");

    let octocrab = octocrab::instance();
    let (from, to) = crate::utils::parse_from_date_and_to_date(from, to)?;
    let new_issues =
        crate::subcribe_issues::get_open_issues_by_date(octocrab, org, repo, from, to).await?;
    let mut link_list = Vec::with_capacity(new_issues.len());
    let mut title_list = Vec::with_capacity(new_issues.len());
    let mut date_list = Vec::with_capacity(new_issues.len());
    for issue in new_issues {
        link_list.push(issue.html_url.to_string());
        title_list.push(format!("**{0}**", issue.title));
        date_list.push(issue.created_at.to_string());
    }
    let mut df = df![
        "created date"  => date_list,
        "title" => title_list,
        "link"  => link_list,
    ]?;
    println!("{repo}'s issues report: {df}");
    create_dir_all(&_path)?;

    let csv_path = format!("{_path}/issue.csv");
    println!("The report has been generated at: {:?}.", csv_path);
    let mut file = File::create(csv_path)?;
    CsvWriter::new(&mut file).finish(&mut df)?;

    Ok(())
}
