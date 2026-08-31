# rust-path-bruter

Fast asynchronous path bruter written in Rust.  
Sends concurrent HTTP requests and reports non-404 responses 
with color-coded status codes.

## Usage

cargo run --release <url> <wordlist> [output.txt]

Example:
cargo run --release https://example.com paths.txt results.txt

## Wordlist

A wordlist file is required — one path per line.  
Recommended source: SecLists by danielmiessler  
https://github.com/danielmiessler/SecLists/tree/master/Discovery/Web-Content

Suggested file: raft-medium-words.txt

## Output

Results are printed to stdout with color-coded HTTP status codes:
- Green  → 2xx (success)
- Yellow → 3xx (redirect)
- Red    → 401/403 (auth required / forbidden)
- Purple → 5xx (server error)

Non-404 and non-400 responses are reported as findings.  
Optionally saves results to a .txt file.

## Dependencies

tokio, reqwest, futures-util, colored, chrono
