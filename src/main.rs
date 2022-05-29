use std::env;
use std::panic::set_hook;

use async_minecraft_ping::ConnectionConfig;
use colored::*;
use futures::future::join_all;
use futures::FutureExt;
use http::Uri;

#[tokio::main]
async fn main() {
    // https://stackoverflow.com/a/51786700/8532605
    // Very jank way to change panic output on release builds
    #[cfg(not(debug_assertions))]
    set_hook(Box::new(|info| {
        if let Some(s) = info.payload().downcast_ref::<&str>() {
            println!("{} {}", "Error:".bold().red(), s);
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            println!("{} {}", "Error:".bold().red(), s);
        }
    }));

    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        panic!("At least 1 server address is required");
    }

    let urls: Vec<Uri> = args[1..]
        .iter()
        .map(|arg| {
            arg.parse::<Uri>()
                .expect(&format!("URL parse failed: {}", arg))
        })
        .collect();

    let config_futures = urls.iter().map(|url| {
        let authority = url.authority().expect(&format!("No URI?: {}", url));
        let mut config = ConnectionConfig::build(authority.host());

        if authority.port().is_some() {
            config = config.with_port(authority.port().unwrap().as_u16());
        }

        config.connect().then(|conn| {
            conn.expect(&format!("Failed to connect to {}", authority.host()))
                .status()
        })
    });

    let pings = join_all(config_futures).await;

    for ping_result in pings {
        match ping_result {
            Ok(status) => println!(
                "{: <40} | {: <10} | {: <10}",
                format!("{}:{}", status.address(), status.port()),
                format!("{} online", status.status.players.online),
                format!("{} max", status.status.players.max)
            ),
            Err(_) => panic!("Failed to get server status"),
        }
    }
}
