#!/usr/bin/env nu

const CODEBERG_USER = "wvhulle-bot"
const TEA_LOGIN_NAME = "codeberg"
const UPDATE_BRANCH = "bot/dependency-updates"
const UPDATE_COMMIT_MESSAGE = "chore(deps): update lock files"
const PULL_REQUEST_DESCRIPTION = "Opened by the weekly dependency update job. The checks on this pull request are the trial build. Commits pushed to this branch by hand stop further automatic updates until it is merged."

def open-pull-request-heads []: nothing -> list<string> {
  ^tea pulls list --login $TEA_LOGIN_NAME --repo $env.CI_REPO --state open --fields head --output simple
  | lines
  | str trim
}

def remote-update-branch-commits []: nothing -> list<string> {
  ^git ls-remote --heads origin $"refs/heads/($UPDATE_BRANCH)"
  | parse "{commit_id}\t{ref_name}"
  | get commit_id
}

def update-branch-has-hand-written-commits []: nothing -> bool {
  ^git fetch --quiet origin $"+refs/heads/($UPDATE_BRANCH):refs/remotes/origin/($UPDATE_BRANCH)"
  ^git log --format=%an $"origin/main..origin/($UPDATE_BRANCH)"
  | lines
  | any {|commit_author| $commit_author != $env.GIT_AUTHOR_NAME }
}

def "main open-dependency-update-pr" [
  --cargo-lock
  --flake-lock
]: nothing -> nothing {
  ^git remote set-url origin $"https://($CODEBERG_USER):($env.CODEBERG_TOKEN)@codeberg.org/($env.CI_REPO).git"
  ^tea login add --name $TEA_LOGIN_NAME --url $env.CI_FORGE_URL --token $env.CODEBERG_TOKEN
  ^git fetch --quiet origin +refs/heads/main:refs/remotes/origin/main
  let remote_commits = remote-update-branch-commits
  let remote_commit = $remote_commits | str join
  let pull_request_is_open = $UPDATE_BRANCH in (open-pull-request-heads)
  if ($remote_commits | is-not-empty) and $pull_request_is_open and (update-branch-has-hand-written-commits) {
    print $"Leaving ($UPDATE_BRANCH) alone because it contains hand-written commits"
    return
  }
  ^git checkout -B $UPDATE_BRANCH origin/main
  if $cargo_lock {
    ^cargo update
  }
  if $flake_lock {
    ^nix flake update
  }
  if (^git diff --name-only | is-empty) {
    print "Dependencies are up to date"
    return
  }
  ^git commit --all --message $UPDATE_COMMIT_MESSAGE
  ^git push $"--force-with-lease=($UPDATE_BRANCH):($remote_commit)" origin $UPDATE_BRANCH
  if $pull_request_is_open {
    print $"Updated the open pull request from ($UPDATE_BRANCH)"
  } else {
    ^tea pulls create --login $TEA_LOGIN_NAME --repo $env.CI_REPO --head $UPDATE_BRANCH --base main --title $UPDATE_COMMIT_MESSAGE --description $PULL_REQUEST_DESCRIPTION
  }
}

def main []: nothing -> nothing {
  print "Run a subcommand, for example: open-dependency-update-pr --cargo-lock --flake-lock"
}
