//! We prefer GitLab Snippets, since GitHub Gists will get blocked for overuse.
//! Fetch the Latest GitLab Snippets / GitHub Gists by User:
//! (1) Process the Build Log
//! (2) Process each Build Target
//! (3) Post to Prometheus Pushgateway

use std::{
    thread::sleep, 
    time::Duration, 
};
use clap::Parser;
use serde_json::Value;

/// Command-Line Arguments
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Init the Logger and Command-Line Args
    env_logger::init();
    // let args = Args::parse();

    // Compose the Prometheus Query
    let query = r##"build_score{config!="leds64_zig", user!="rewind", user!="nuttxlinux", user!="nuttxmacos", user!="jerpelea"} < 0.5"##;
    println!("query={query}");
    let params = [("query", query)];
    let client = reqwest::Client::new();
    let prometheus = "http://localhost:9090/api/v1/query";
    let res = client
        .post(prometheus)
        .form(&params)
        .send()
        .await?;
    println!("res={res:?}");
    if !res.status().is_success() {
        println!("*** Pushgateway Failed");
        sleep(Duration::from_secs(1));
    }
    println!("Status: {}", res.status());
    println!("Headers:\n{:#?}", res.headers());
    let body = res.text().await?;
    println!("Body: {body}");
    let data: Value = serde_json::from_str(&body)?;
    let builds = &data["data"]["result"];
    println!("\n\nbuilds={builds:?}");

    // Process Each Build
    for build in builds.as_array().unwrap() {
        println!("\n\nbuild={build:?}");
        let metric = &build["metric"];
        println!("\n\nmetric={metric:?}");
    }

    // Fetch the Latest Snippets, reverse chronological order
    // let client = reqwest::Client::new();
    // let url = format!("https://gitlab.com/api/v4/projects/{user}%2F{repo}/snippets?per_page=100&page=1");
    // let res = client
    //     .get(url)
    //     .header("PRIVATE-TOKEN", token)
    //     .send()
    //     .await?;
    // println!("Status: {}", res.status());
    // println!("Headers:\n{:#?}", res.headers());
    // let body = res.text().await?;
    // println!("Body: {body}");
    // let snippets: Value = serde_json::from_str(&body)?;

    // Wait a while
    sleep(Duration::from_secs(1));

    // Return OK
    Ok(())
}
