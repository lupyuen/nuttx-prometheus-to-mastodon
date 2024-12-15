//! (1) Fetch the Failed NuttX Builds from Prometheus
//! (2) Post to Mastodon

use std::{
    fs::File,
    io::{BufReader, Write},
    thread::sleep,
    time::Duration,
};
use clap::Parser;
use serde_json::{
    json,
    to_string_pretty,
    Value,
};

// Remembers the Mastodon Posts for All Builds:
// {
//   "rv-virt:citest" : {
//     status_id: "12345",
//     users: ["nuttxpr", "NuttX", "lupyuen"]
//   }
//   "rv-virt:citest64" : ...
// }
const ALL_BUILDS_FILENAME: &str = "/tmp/nuttx-prometheus-to-mastodon.json";

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
    let query = r##"
        build_score{
            config!="leds64_zig",
            user!="rewind",
            user!="nuttxlinux",
            user!="nuttxmacos",
            user!="jerpelea"
        } < 0.5
    "##;
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
        println!("*** Prometheus Failed");
        sleep(Duration::from_secs(1));
    }
    // println!("Status: {}", res.status());
    // println!("Headers:\n{:#?}", res.headers());
    let body = res.text().await?;
    // println!("Body: {body}");
    let data: Value = serde_json::from_str(&body).unwrap();
    let builds = &data["data"]["result"];
    // println!("\nbuilds={builds:?}");

    // Load the Mastodon Posts for All Builds
    let mut all_builds = json!({});
    if let Ok(file) = File::open(ALL_BUILDS_FILENAME) {
        let reader = BufReader::new(file);
        all_builds = serde_json::from_reader(reader).unwrap();
        println!("\nall_builds=\n{}", to_string_pretty(&all_builds).unwrap());
    }

    // For Each Failed Build...
    for build in builds.as_array().unwrap() {
        println!("build=\n{}", to_string_pretty(build).unwrap());
        let metric = &build["metric"];
        println!("metric=\n{}", to_string_pretty(metric).unwrap());
        let board = metric["board"].as_str().unwrap();
        let config = metric["config"].as_str().unwrap();
        let user = metric["user"].as_str().unwrap();
        let msg = metric["msg"].as_str().unwrap_or("");
        let config_upper = config.to_uppercase();
        let target = format!("{board}:{config}");
        println!("board={board}");
        println!("config={config}");
        println!("user={user}");
        println!("msg=\n<<\n{msg}\n>>");

        // Compose the Mastodon Post as...
        // rv-virt : CITEST - Build Failed (NuttX)
        // NuttX Dashboard: ...
        // Build History: ...
        // [Error Message]
        let mut status = format!(
            r##"
{board} : {config_upper} - Build Failed ({user})
NuttX Dashboard: https://nuttx-dashboard.org
Build History: https://nuttx-dashboard.org/d/fe2q876wubc3kc/nuttx-build-history?var-board={board}&var-config={config}

{msg}
            "##);
        status.truncate(512);  // Mastodon allows only 500 chars
        let mut params = Vec::new();
        params.push(("status", status));

        // If the Mastodon Post already exists for Board and Config:
        // Reply to the Mastodon Post
        if let Some(status_id) = all_builds[&target]["status_id"].as_str() {
            params.push(("in_reply_to_id", status_id.to_string()));

            // If the User already exists for the Board and Config:
            // Skip the Mastodon Post
            if let Some(users) = all_builds[&target]["users"].as_array() {
                if users.contains(&json!(user)) {
                    println!("Skipping {user} @ {target}, already exists\n");
                    continue;
                }
            }
        }

        // Post to Mastodon
        let token = std::env::var("MASTODON_TOKEN")
            .expect("MASTODON_TOKEN env variable is required");
        let client = reqwest::Client::new();
        let mastodon = "https://nuttx-feed.org/api/v1/statuses";
        let res = client
            .post(mastodon)
            .header("Authorization", format!("Bearer {token}"))
            .form(&params)
            .send()
            .await?;
        println!("res={res:?}");
        if !res.status().is_success() {
            println!("*** Mastodon Failed: {user} @ {target}");
            sleep(Duration::from_secs(30));
            continue;
        }
        // println!("Status: {}", res.status());
        // println!("Headers:\n{:#?}", res.headers());
        let body = res.text().await?;
        println!("Body: {body}");

        // Remember the Mastodon Post ID (Status ID)
        let status: Value = serde_json::from_str(&body).unwrap();
        let status_id = status["id"].as_str().unwrap();
        println!("status_id={status_id}");
        all_builds[&target]["status_id"] = status_id.into();

        // Append the User to All Builds
        if let Some(users) = all_builds[&target]["users"].as_array() {
            if !users.contains(&json!(user)) {
                let mut users = users.clone();
                users.push(json!(user));
                all_builds[&target]["users"] = json!(users);
            }
        } else {
            all_builds[&target]["users"] = json!([user]);
        }

        // Save the Mastodon Posts for All Builds
        let json = to_string_pretty(&all_builds).unwrap();
        let mut file = File::create(ALL_BUILDS_FILENAME).unwrap();
        file.write_all(json.as_bytes()).unwrap();
        println!("\nall_builds=\n{json}\n");

        // Wait a while
        sleep(Duration::from_secs(30));
    }

    // Return OK
    Ok(())
}
