use colored::Colorize;
use futures_util::stream::{self, StreamExt};
use std::env;
use std::fs;
use std::io::Write;
use std::time::Instant;

async fn check_path(
    client: reqwest::Client,
    base_url: String,
    path: String,
) -> Option<(String, u16)> {
    let url = format!(
        "{}/{}",
        base_url.trim_end_matches('/'),
        path.trim_start_matches('/')
    );

    match client.get(&url).send().await {
        Ok(resp) => {
            let status = resp.status().as_u16();
            match status {
                404 | 400 => None,
                _ => Some((url, status)),
            }
        }
        Err(_) => None,
    }
}

fn status_color(status: u16) -> String {
    match status {
        200..=299 => format!("[{}]", status).green().to_string(),
        301..=399 => format!("[{}]", status).yellow().to_string(),
        401 | 403 => format!("[{}]", status).red().to_string(),
        500..=599 => format!("[{}]", status).magenta().to_string(),
        _ => format!("[{}]", status).white().to_string(),
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!(
            "{}",
            "Usage: path-bruter <url> <wordlist> [output.txt]".red()
        );
        println!(
            "{}",
            "Example: path-bruter https://app.ferrero.com paths.txt results.txt".yellow()
        );
        return;
    }

    let base_url = args[1].clone();
    let wordlist_path = args[2].clone();
    let output_path = args.get(3).cloned();

    let content = match fs::read_to_string(&wordlist_path) {
        Ok(c) => c,
        Err(e) => {
            println!("{} {}", "Error reading wordlist:".red(), e);
            return;
        }
    };

    let wordlist: Vec<String> = content
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
        .map(|l| l.trim().to_string())
        .collect();

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("Mozilla/5.0-VDP-ferrero-international-s.a-lurbik")
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();

    let start = Instant::now();
    let now = chrono::Local::now();

    println!("{}", "─".repeat(60).cyan());
    println!(
        "{}",
        format!(
            "  Path Bruter | Target: {} | {}",
            base_url,
            now.format("%Y-%m-%d %H:%M:%S")
        )
        .cyan()
    );
    println!(
        "{}",
        format!("  Wordlist: {} paths | Concurrency: 20", wordlist.len()).cyan()
    );
    println!("{}", "─".repeat(60).cyan());

    let concurrency = 20;

    let results: Vec<Option<(String, u16)>> = stream::iter(wordlist)
        .map(|path| {
            let client = client.clone();
            let base_url = base_url.clone();
            async move { check_path(client, base_url, path).await }
        })
        .buffer_unordered(concurrency)
        .collect()
        .await;

    let found: Vec<(String, u16)> = results.into_iter().flatten().collect();

    for (url, status) in &found {
        println!(
            "{} {} {}",
            "[+]".green().bold(),
            status_color(*status),
            url.green()
        );
    }

    let elapsed = start.elapsed();
    println!("{}", "─".repeat(60).cyan());
    println!(
        "{}",
        format!(
            "  Found: {} | Time: {:.2}s",
            found.len(),
            elapsed.as_secs_f64()
        )
        .cyan()
    );
    println!("{}", "─".repeat(60).cyan());

    if let Some(path) = output_path {
        let mut file = fs::File::create(&path).expect("Cannot create output file");
        writeln!(file, "Path Bruter Results").unwrap();
        writeln!(file, "Target: {}", base_url).unwrap();
        writeln!(file, "Date: {}", now.format("%Y-%m-%d %H:%M:%S")).unwrap();
        writeln!(file, "{}", "─".repeat(60)).unwrap();
        for (url, status) in &found {
            writeln!(file, "[{}] {}", status, url).unwrap();
        }
        writeln!(file, "{}", "─".repeat(60)).unwrap();
        writeln!(
            file,
            "Found: {} paths in {:.2}s",
            found.len(),
            elapsed.as_secs_f64()
        )
        .unwrap();
        println!("{} {}", "Output saved to:".cyan(), path.green());
    }
}
