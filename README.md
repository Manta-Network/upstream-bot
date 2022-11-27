# Upstream-subscriber-bot
A discord bot for subscribing web3 projects' pull requessts and issues, and send them to your discord channels.

## Configuration
### Configure discord token
Please take a look at the section `discord` in [config](./config.toml).

### When to trigger the subscription
Please take a look at the section `schedule` in [config](./config.toml).

### Subscribe a new repository

Please take a look at [config](./config.toml).
```toml
[organization.repository]
organization = "organization-name"
repository = "repository-name"
query-release = false
```

For example, if you wanto to subscribe [go-ethereum](https://github.com/ethereum/go-ethereum), append the following section to [config](./config.toml).
```toml
[ethereum.repository]
organization = "ethereum"
repository = "go-ethereum"
query-release = true
```

## Command Line

How to generate a weekly update
- Issue

Only query **created** issues.
```shell
cargo r --release issue --org=paritytech --repo=substrate --from=2022-11-18 --to=2022-11-25 --status=open
```

The result should be like this
```
┌─────────────────────────┬─────────────────────────────────────┬─────────────────────────────────────┐
│ created date            ┆ title                               ┆ link                                │
│ ---                     ┆ ---                                 ┆ ---                                 │
│ str                     ┆ str                                 ┆ str                                 │
╞═════════════════════════╪═════════════════════════════════════╪═════════════════════════════════════╡
│ 2022-11-25 14:45:34 UTC ┆ **client/mmr: make it resilient ... ┆ https://github.com/paritytech/su... │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ 2022-11-23 20:41:15 UTC ┆ **PoV Limit Pallet**                ┆ https://github.com/paritytech/su... │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ 2022-11-23 14:37:39 UTC ┆ **Add warp-sync `zombienet` test... ┆ https://github.com/paritytech/su... │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ 2022-11-22 11:02:15 UTC ┆ **Use the new `desired_targets_c... ┆ https://github.com/paritytech/su... │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
```

- Pull Request

Only query **merged** PRs.
```shell
cargo r pr --org=paritytech --repo=substrate --from=2022-11-18 --to=2022-11-25 --status=merged
```

The result should be like this
```
┌─────────────────────────┬─────────────────────────────────────┬─────────────────────────────────────┐
│ date                    ┆ title                               ┆ link                                │
│ ---                     ┆ ---                                 ┆ ---                                 │
│ str                     ┆ str                                 ┆ str                                 │
╞═════════════════════════╪═════════════════════════════════════╪═════════════════════════════════════╡
│ 2022-11-23 17:32:59 UTC ┆ **Explicitly unset RUSTC_WRAPPER... ┆ https://github.com/paritytech/su... │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ 2022-11-24 05:05:54 UTC ┆ **add EnsureWithSuccess**           ┆ https://github.com/paritytech/su... │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ 2022-11-23 16:47:35 UTC ┆ **Add total nb to trie migration... ┆ https://github.com/paritytech/su... │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
```

More detail, please
```
cargo r -- -h
```

> Remember, github has [rate limit](https://docs.github.com/en/rest/overview/resources-in-the-rest-api#rate-limiting) for unauthenticated requests, it's `60 requests per hour`.
> To find out how many requests you have sent, try this command:
> ```
> curl -I https://api.github.com/users/octocat
> ```
> You will see the result.

## Tips
### Key Format in Sled

#### Issue

key format for open issue:
```
organization#repository#issues#open#issue_number
```
Example:
```
parity#substrate#issues#open#100
```

key format for closed issue:
```
organization#repository#issues#closed#issue_number
```

Example:
```
parity#substrate#issues#closed#99
```

#### Pull Request

Key format for open pr:
```
organization#repository#prs#open#pr_number
```
Example:
```
parity#substrate#prs#open#100
```

Key format for merged pr:
```
organization#repository#prs#merged#pr_number
```

Example:
```
parity#substrate#prs#merged#99
```

Key format for closed pr:
```
organization#repository#prs#closed#pr_number
```

Example:
```
parity#substrate#prs#closed#99
```

With such key format, for example, it's very easy to get all open issues like this:
```rust
let key_prefix = "parity#substrate#issues#open";
let mut iter = db.scan_prefix(key_prefix);
while let Some(Ok((key, val))) = iter.next() {
    let issue = serde_json::from_slice(val.as_ref())?;
    ...
}
```

Or get all issues by this key prefix, whatever it's open or closed.
```rust
let key_prefix = "parity#substrate#issues";
...
```