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

use crate::types::{Issue, LatestRelease, PullRequest, Repository};
use crate::utils::{get_discord_token, get_repositories, get_update_frequence};
use crate::{subcribe_issues, subcribe_prs, subcribe_releases};
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
    utils::MessageBuilder,
};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use toml::Value;

pub struct BotHandler {
    frequence: Duration,
    db: Arc<sled::Db>,
    repositories: Vec<Repository>,
}

impl BotHandler {
    pub fn new(secs: u64, db: sled::Db, repos: Vec<Repository>) -> Self {
        Self {
            frequence: Duration::from_secs(secs),
            db: Arc::new(db),
            repositories: repos,
        }
    }
}

#[async_trait]
impl EventHandler for BotHandler {
    async fn message(&self, context: Context, msg: Message) {
        let _channel = match msg.channel_id.to_channel(&context).await {
            Ok(channel) => channel,
            Err(why) => {
                println!("Error getting channel: {:?}", why);
                return;
            }
        };

        // When to query upstream changes.
        sleep(self.frequence).await;

        for repo in self.repositories.iter() {
            // Query issues first.
            if let Ok(new_issues) = subcribe_issues::get_new_issues(
                self.db.clone(),
                &repo.organization,
                &repo.repository,
            )
            .await
            {
                for issue in new_issues.iter() {
                    handle_issue_message(&repo.repository, Some(issue), &msg, &context).await;
                }
            } else {
                handle_issue_message(&repo.repository, None, &msg, &context).await;
            }

            // Query PRs then.
            if let Ok(new_prs) = subcribe_prs::get_new_pull_requests(
                self.db.clone(),
                &repo.organization,
                &repo.repository,
            )
            .await
            {
                for pr in new_prs.iter() {
                    handle_pr_message(&repo.repository, Some(pr), &msg, &context).await;
                }
            } else {
                handle_pr_message(&repo.repository, None, &msg, &context).await;
            }

            // Query latest release then.
            if let Ok(latest_release) =
                subcribe_releases::get_latest_release(&repo.organization, &repo.repository).await
            {
                let latest_release = LatestRelease::from(latest_release);
                handle_release_message(&repo.repository, Some(&latest_release), &msg, &context)
                    .await;
            } else {
                handle_release_message(&repo.repository, None, &msg, &context).await;
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

async fn handle_issue_message(repo: &str, issue: Option<&Issue>, msg: &Message, context: &Context) {
    if let Some(issue) = issue {
        /*
            The example of message format:
            **Substrate Issue**: issue's title:
            issue's url
        */
        let response = MessageBuilder::new()
            .push_bold_safe(repo)
            .push(" **Issue**: ")
            .push(&issue.title)
            .push(" ")
            .push(issue.url.as_str())
            .build();

        if let Err(why) = msg.channel_id.say(&context.http, &response).await {
            println!("Error sending message: {:?}", why);
        }
    } else {
        let response = MessageBuilder::new()
            .push("Failed to query new issue From ")
            .push_bold_safe(repo)
            .build();

        if let Err(why) = msg.channel_id.say(&context.http, &response).await {
            println!("Error sending message: {:?}", why);
        }
    }
}

async fn handle_pr_message(repo: &str, pr: Option<&PullRequest>, msg: &Message, context: &Context) {
    if let Some(pr) = pr {
        /*
            The example of message format:
            **Substrate PR**: pr's title:
            pr's url
        */
        let response = MessageBuilder::new()
            .push_bold_safe(repo)
            .push(" **PR**: ")
            .push(pr.title.as_deref().unwrap_or("No title"))
            .push(" ")
            .push(pr.url.as_ref().map(|u| u.as_str()).unwrap_or("No url"))
            .build();

        if let Err(why) = msg.channel_id.say(&context.http, &response).await {
            println!("Error sending message: {:?}", why);
        }
    } else {
        let response = MessageBuilder::new()
            .push("Failed to query merged PRs From ")
            .push_bold_safe(repo)
            .build();

        if let Err(why) = msg.channel_id.say(&context.http, &response).await {
            println!("Error sending message: {:?}", why);
        }
    }
}

async fn handle_release_message(
    repo: &str,
    release: Option<&LatestRelease>,
    msg: &Message,
    context: &Context,
) {
    if let Some(release) = release {
        /*
            The example of message format:
            **Polkadot Latest Release**: Release Title:
            release's url
        */
        let response = MessageBuilder::new()
            .push_bold_safe(repo)
            .push(" **Latest Release**: ")
            .push(
                release
                    .release_name
                    .as_deref()
                    .unwrap_or("No release title"),
            )
            .push(" ")
            .push(release.url.as_str())
            .build();

        if let Err(why) = msg.channel_id.say(&context.http, &response).await {
            println!("Error sending message: {:?}", why);
        }
    } else {
        let response = MessageBuilder::new()
            .push("Failed to query latest release from ")
            .push_bold_safe(repo)
            .build();

        if let Err(why) = msg.channel_id.say(&context.http, &response).await {
            println!("Error sending message: {:?}", why);
        }
    }
}

pub async fn discord_bot(config: &Value) {
    // Get discord bot token.
    let token = get_discord_token(config);
    // get db handler
    let db = crate::utils::db_config().expect("Failed to create or open db.");
    // Get the frequence of querying upstream.
    let frequence = get_update_frequence(config) as u64;

    // Get all repositories
    let repositories = get_repositories(config).expect("Failed to get all repositories.");

    // configure bot handler
    let bot_handler = BotHandler::new(frequence, db, repositories);

    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&token, intents)
        .event_handler(bot_handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_foramt() {
        let repo = "Substrate";
        let title = "Improve JSON error reporting";
        let url = "https://github.com/XAMPPRocky/octocrab/issues/13";
        let response = MessageBuilder::new()
            .push_bold_safe(repo)
            .push(" **PR**: ")
            .push(title)
            .push(" ")
            .push(url)
            .build();
        assert_eq!(
            response,
            "**Substrate** **PR**: Improve JSON error reporting https://github.com/XAMPPRocky/octocrab/issues/13"
        );
    }
}
