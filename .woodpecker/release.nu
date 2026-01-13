#!/usr/bin/env nu

# Create a release on Codeberg
# Usage: ./release.nu <tag> [--draft] [--dry-run]

def main [
    tag: string       # Tag name (e.g., v0.0.119)
    --draft           # Create as draft release
    --dry-run         # Show what would be uploaded without publishing
]: nothing -> nothing {
    let token = load-token

    if $token == "" and not $dry_run {
        error make { msg: "CODEBERG_TOKEN not set. Create .env file or set environment variable." }
    }

    if not (dist | path exists) {
        error make { msg: "dist/ directory not found. Run build.nu first." }
    }

    let files = ls dist | get name

    let notes_file = RELEASE_NOTES.md
    if not ($notes_file | path exists) {
        error make { msg: $"($notes_file) not found. Run changelog.nu first." }
    }
    let notes = open $notes_file

    if $dry_run {
        print-dry-run $tag $draft $files $notes
        return
    }

    print $"(ansi blue)Creating release ($tag)...(ansi reset)"

    let repo_path = get-repo-path
    let api_url = $"https://codeberg.org/api/v1/repos/($repo_path)/releases"

    let body = {
        tag_name: $tag
        name: $tag
        body: $notes
        draft: $draft
        prerelease: false
    }

    let response = try {
        http post $api_url ($body | to json) --content-type application/json --headers [Authorization $"token ($token)"]
    } catch { |e|
        error make { msg: $"Failed to create release: ($e.msg)" }
    }

    let release_id = $response.id
    print $"(ansi green)Created release with ID ($release_id)(ansi reset)"

    for f in $files {
        print $"(ansi blue)Uploading ($f)...(ansi reset)"
        let filename = $f | path basename
        let upload_url = $"https://codeberg.org/api/v1/repos/($repo_path)/releases/($release_id)/assets?name=($filename)"

        try {
            http post $upload_url (open --raw $f) --content-type application/octet-stream --headers [Authorization $"token ($token)"]
        } catch { |e|
            error make { msg: $"Failed to upload ($f): ($e.msg)" }
        }
    }

    print $"(ansi green)Release ($tag) published!(ansi reset)"
    print $"https://codeberg.org/($repo_path)/releases/tag/($tag)"
}

def load-token []: nothing -> string {
    if (.env | path exists) {
        open .env
            | lines
            | parse "{key}={value}"
            | where key == CODEBERG_TOKEN
            | get value
            | first
            | default ""
    } else {
        $env.CODEBERG_TOKEN? | default ""
    }
}

def get-repo-path []: nothing -> string {
    let remote = if (^git remote | complete | get stdout | lines | any { $in == codeberg }) {
        ^git remote get-url codeberg | complete | get stdout | str trim
    } else {
        ^git remote get-url origin | complete | get stdout | str trim
    }

    $remote
        | str replace https://codeberg.org/ ""
        | str replace ssh://git@codeberg.org/ ""
        | str replace git@codeberg.org: ""
        | str replace .git ""
}

def print-dry-run [
    tag: string
    draft: bool
    files: list<string>
    notes: string
]: nothing -> nothing {
    print $"(ansi yellow)Dry run - would create release:(ansi reset)"
    print $"  Tag: ($tag)"
    print $"  Draft: ($draft)"
    print "  Files:"
    for f in $files {
        let size = (ls $f).0.size
        print $"    - ($f) \(($size)\)"
    }
    print "  Notes:"
    print $notes
}
