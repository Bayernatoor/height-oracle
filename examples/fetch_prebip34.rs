use std::collections::HashMap;
use std::env;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::PathBuf;

use height_oracle::BIP34_ACTIVATION_HEIGHT;

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct JsonRpcRequest<'a, T> {
    jsonrpc: &'a str,
    #[serde(rename = "id")]
    _id: String,
    method: &'a str,
    params: T,
}

#[derive(Deserialize)]
struct JsonRpcResponse<T> {
    result: Option<T>,
    error: Option<JsonRpcError>,
    #[serde(rename = "id")]
    _id: serde_json::Value,
}

#[derive(Deserialize)]
struct JsonRpcError {
    code: i64,
    message: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Defaults
    let mut rpc_url = String::from("http://127.0.0.1:8332");
    let mut rpc_user: Option<String> = env::var("BTC_RPC_USER").ok();
    let mut rpc_pass: Option<String> = env::var("BTC_RPC_PASS").ok();
    let mut cookie_path: Option<PathBuf> = None;
    let mut concurrency: usize = 32;

    let default_end: u32 = BIP34_ACTIVATION_HEIGHT - 1; // inclusive end height
    let mut start_height: u32 = 0;
    let mut end_height: u32 = default_end;

    let mut output_path = PathBuf::from("assets/prebip34.txt");

    // Parse simple CLI flags
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--rpc-url" => {
                if let Some(v) = args.next() {
                    rpc_url = v;
                }
            }
            "--rpc-user" => {
                if let Some(v) = args.next() {
                    rpc_user = Some(v);
                }
            }
            "--rpc-pass" => {
                if let Some(v) = args.next() {
                    rpc_pass = Some(v);
                }
            }
            "--cookie" => {
                if let Some(v) = args.next() {
                    cookie_path = Some(PathBuf::from(v));
                }
            }
            "--concurrency" => {
                if let Some(v) = args.next() {
                    concurrency = v.parse().unwrap_or(concurrency);
                }
            }
            "--start-height" => {
                if let Some(v) = args.next() {
                    start_height = v.parse().unwrap_or(start_height);
                }
            }
            "--end-height" => {
                if let Some(v) = args.next() {
                    end_height = v.parse().unwrap_or(end_height);
                }
            }
            "--output" => {
                if let Some(v) = args.next() {
                    output_path = PathBuf::from(v);
                }
            }
            _ => {}
        }
    }

    if end_height < start_height {
        eprintln!("end-height must be >= start-height");
        std::process::exit(1);
    }

    // If user/pass not provided, try cookie at default path
    if rpc_user.is_none() || rpc_pass.is_none() {
        let default_cookie = dirs::home_dir().map(|h| h.join(".bitcoin/.cookie"));
        let cookie_file = cookie_path.or(default_cookie);
        if let Some(path) = cookie_file {
            if path.exists() {
                let content = std::fs::read_to_string(&path)?;
                // cookie content is "user:password"
                if let Some((u, p)) = content.trim().split_once(':') {
                    rpc_user = Some(u.to_string());
                    rpc_pass = Some(p.to_string());
                }
            }
        }
    }

    let (rpc_user, rpc_pass) = match (rpc_user, rpc_pass) {
        (Some(u), Some(p)) => (u, p),
        _ => {
            eprintln!("Missing RPC credentials. Provide --rpc-user/--rpc-pass, set BTC_RPC_USER/BTC_RPC_PASS, or ensure ~/.bitcoin/.cookie exists.");
            std::process::exit(1);
        }
    };

    // HTTP client
    let client = reqwest::Client::builder().build()?;

    let total: u64 = (end_height as u64) - (start_height as u64) + 1;
    println!(
        "Fetching pre-BIP34 block hashes: heights {}..={} ({} blocks) with concurrency={}",
        start_height, end_height, total, concurrency
    );
    println!("RPC URL: {}", rpc_url);

    // Prepare heights list
    let heights: Vec<u32> = (start_height..=end_height).collect();

    use futures::{stream, StreamExt};

    #[derive(Clone)]
    struct Auth {
        user: String,
        pass: String,
    }
    let auth = Auth {
        user: rpc_user,
        pass: rpc_pass,
    };

    // Concurrently fetch hashes
    let results = stream::iter(heights.clone())
        .map(|h| {
            let client = client.clone();
            let url = rpc_url.clone();
            let auth = auth.clone();
            async move {
                // Build JSON-RPC request
                let req_body = JsonRpcRequest {
                    jsonrpc: "1.0",
                    _id: format!("getblockhash-{}", h),
                    method: "getblockhash",
                    params: vec![serde_json::Value::from(h as u64)],
                };

                // Send request with basic auth
                let resp = client
                    .post(&url)
                    .basic_auth(&auth.user, Some(&auth.pass))
                    .json(&req_body)
                    .send()
                    .await;

                let resp = match resp {
                    Ok(r) => r,
                    Err(e) => return Err((h, format!("request error: {}", e))),
                };

                let status = resp.status();
                let text = resp.text().await.map_err(|e| (h, e.to_string()))?;
                if !status.is_success() {
                    return Err((h, format!("HTTP {}: {}", status, text)));
                }

                let parsed: JsonRpcResponse<String> = serde_json::from_str(&text)
                    .map_err(|e| (h, format!("decode error: {} - body: {}", e, text)))?;

                if let Some(err) = parsed.error {
                    return Err((h, format!("RPC error {}: {}", err.code, err.message)));
                }
                let hash = parsed
                    .result
                    .ok_or_else(|| (h, String::from("missing result")))?;

                // Fetch block header to inspect version. If version == 2, write a placeholder 'x'
                let header_req = JsonRpcRequest {
                    jsonrpc: "1.0",
                    _id: format!("getblockheader-{}", h),
                    method: "getblockheader",
                    params: vec![
                        serde_json::Value::from(hash.clone()),
                        serde_json::Value::from(true),
                    ],
                };

                let resp2 = client
                    .post(&url)
                    .basic_auth(&auth.user, Some(&auth.pass))
                    .json(&header_req)
                    .send()
                    .await;

                let resp2 = match resp2 {
                    Ok(r) => r,
                    Err(e) => return Err((h, format!("header request error: {}", e))),
                };

                let status2 = resp2.status();
                let text2 = resp2.text().await.map_err(|e| (h, e.to_string()))?;
                if !status2.is_success() {
                    return Err((h, format!("HTTP {}: {}", status2, text2)));
                }

                let parsed2: JsonRpcResponse<serde_json::Value> = serde_json::from_str(&text2)
                    .map_err(|e| (h, format!("decode header error: {} - body: {}", e, text2)))?;

                if let Some(err) = parsed2.error {
                    return Err((h, format!("RPC header error {}: {}", err.code, err.message)));
                }

                let use_x = parsed2
                    .result
                    .as_ref()
                    .and_then(|v| v.get("version"))
                    .and_then(|ver| ver.as_i64())
                    .map(|ver| ver == 2)
                    .unwrap_or(false);

                if use_x {
                    Ok::<(u32, String), (u32, String)>((h, "x".to_string()))
                } else {
                    Ok::<(u32, String), (u32, String)>((h, hash))
                }
            }
        })
        .buffer_unordered(concurrency)
        .collect::<Vec<_>>()
        .await;

    // Partition successes and failures
    let mut by_height: HashMap<u32, String> = HashMap::with_capacity(results.len());
    let mut failures: Vec<(u32, String)> = Vec::new();
    for r in results {
        match r {
            Ok((h, hash)) => {
                by_height.insert(h, hash);
            }
            Err((h, err)) => {
                failures.push((h, err));
            }
        }
    }

    if !failures.is_empty() {
        eprintln!(
            "Failed to fetch {} heights (showing up to 10):",
            failures.len()
        );
        for (i, (h, err)) in failures.iter().take(10).enumerate() {
            eprintln!("  {}. height {}: {}", i + 1, h, err);
        }
        eprintln!("You can re-run with a lower --concurrency or check your node's rpcworkqueue/rpcthreads settings.");
        std::process::exit(1);
    }

    // Ensure assets directory exists
    if let Some(parent) = output_path.parent() {
        create_dir_all(parent)?;
    }

    // Write in order
    let mut file = File::create(&output_path)?;
    for h in start_height..=end_height {
        let hash = by_height.get(&h).expect("missing height in map");
        writeln!(file, "{}", hash)?;
    }

    println!("Wrote {} lines to {}", total, output_path.display());
    println!("Done.");

    Ok(())
}
