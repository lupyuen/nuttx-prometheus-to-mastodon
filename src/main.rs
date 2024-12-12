//! (1) Fetch the Failed NuttX Builds from Prometheus
//! (2) Post to Mastodon

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

    // Fetch the Failed Builds from Prometheus
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

    // Process Each Failed Build
    for build in builds.as_array().unwrap() {
        println!("\n\nbuild={build:?}");
        let metric = &build["metric"];
        println!("\n\nmetric={metric:?}");
        let board = &metric["board"].as_str().unwrap();
        let config = &metric["config"].as_str().unwrap();
        let config_upper = config.to_uppercase();
        let user = &metric["user"].as_str().unwrap();
        let msg = &metric["msg"].as_str().unwrap();
        println!("\n\nboard={board}");
        println!("config={config}");
        println!("user={user}");
        println!("msg={msg}");

        // Format post as...
        // rv-virt : CITEST - Build Failed (NuttX)
        // NuttX Dashboard: ...
        // Build History: ...
        // [Error Message]
        let status = &format!(
            r##"
{board} : {config_upper} - Build Failed ({user})
NuttX Dashboard: https://nuttx-dashboard.org
Build History: https://nuttx-dashboard.org/d/fe2q876wubc3kc/nuttx-build-history?var-board={board}&var-config={config}

{msg}
            "##)
            [..450];  // Mastodon allows only 500 chars

        // Post to Mastodon
        let token = std::env::var("MASTODON_TOKEN")
            .expect("MASTODON_TOKEN env variable is required");
        let params = [("status", status)];
        let client = reqwest::Client::new();
        let mastodon = "https://nuttx-feed.org/api/v1/statuses";
        let res = client
            .post(mastodon)
            .header("Authorization", format!("Bearer {token}"))
            .form(&params)
            .send()
            .await?;
        println!("Status: {}", res.status());
        println!("Headers:\n{:#?}", res.headers());
        let body = res.text().await?;
        println!("Body: {body}");

        std::process::exit(0);

        // Wait a while
        sleep(Duration::from_secs(1));
    }

    // Return OK
    Ok(())
}
