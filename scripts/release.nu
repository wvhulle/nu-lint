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
        error make {msg: "CODEBERG_TOKEN not set. Add secret in Woodpecker CI settings or create .env file locally."}
    }

    if $token != "" {
        print $"(ansi green)Token loaded \(($token | str length) chars\)(ansi reset)"
    }

    if not ("dist" | path exists) {
        error make {msg: "dist/ directory not found. Run build.nu first."}
    }

    let files = try { ls dist | get name } catch { [] }

    let notes_file = "RELEASE_NOTES.md"
    if not ($notes_file | path exists) {
        error make {msg: $"($notes_file) not found. Run changelog.nu first."}
    }
    let notes = try { open $notes_file } catch { "" }

    if $dry_run {
        print-dry-run {tag: $tag, draft: $draft, files: $files, notes: $notes}
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
    } catch {|e|
        error make {msg: $"Failed to create release: ($e.msg)"}
    }

    let release_id = $response.id
    print $"(ansi green)Created release with ID ($release_id)(ansi reset)"

    for f in $files {
        print $"(ansi blue)Uploading ($f)...(ansi reset)"
        let filename = $f | path basename
        let upload_url = $"https://codeberg.org/api/v1/repos/($repo_path)/releases/($release_id)/assets?name=($filename)"

        let content = try { open --raw $f } catch {|e|
            error make {msg: $"Failed to read ($f): ($e.msg)"}
        }
        try {
            http post $upload_url $content --content-type application/octet-stream --headers [Authorization $"token ($token)"]
        } catch {|e|
            error make {msg: $"Failed to upload ($f): ($e.msg)"}
        }
    }

    print $"(ansi green)Release ($tag) published!(ansi reset)\nhttps://codeberg.org/($repo_path)/releases/tag/($tag)"
}

def load-token []: nothing -> string {
    # CI environment variable takes priority
    let env_token = $env.CODEBERG_TOKEN? | default ""
    if $env_token != "" {
        return $env_token
    }

    # Fall back to .env file for local development
    if (".env" | path exists) {
        try {
            open .env
                | lines
                | parse "{key}={value}"
                | where key == CODEBERG_TOKEN
                | get value
                | first
                | default ""
        } catch { "" }
    } else {
        ""
    }
}

def get-repo-path []: nothing -> string {
    let result = git remote | complete
    let has_codeberg = if $result.exit_code == 0 {
        $result.stdout | lines | any { $in == codeberg }
    } else {
        false
    }

    let remote = if $has_codeberg {
        let r = git remote get-url codeberg | complete
        if $r.exit_code == 0 { $r.stdout | str trim } else { "" }
    } else {
        let r = git remote get-url origin | complete
        if $r.exit_code == 0 { $r.stdout | str trim } else { "" }
    }

    $remote
        | str replace --all --regex 'https://codeberg.org/|ssh://git@codeberg.org/|git@codeberg.org:|\.git$' ""
}

def print-dry-run [
    opts: record<tag: string, draft: bool, files: list<string>, notes: string>
]: nothing -> nothing {
    print $"(ansi yellow)Dry run - would create release:(ansi reset)
  Tag: ($opts.tag)
  Draft: ($opts.draft)"
    print "  Files:"
    for f in $opts.files {
        let size = try { ls $f | get 0?.size? | default "?" } catch { "?" }
        print $"    - ($f) \(($size)\)"
    }
    print $"  Notes:\n($opts.notes)"
}
