#![feature(async_closure)]
use io_plugin_example::{Error, ExamplePluginHandle};
use lazy_static::lazy_static;
use regex::Regex;
use std::{error::Error as StdError, process::Stdio as StdioBehaviour};
use tokio::{
    io::{stdin, AsyncBufReadExt, BufReader},
    main,
    process::Command,
};

#[main]
async fn main() {
    let mut plugin = (async || -> Result<_, Box<dyn StdError>> {
        let process = Command::new("target/debug/plugin-example")
            .stdin(StdioBehaviour::piped())
            .stdout(StdioBehaviour::piped())
            .spawn()?;
        Ok(ExamplePluginHandle::new(process).await?)
    })()
    .await
    .unwrap();
    println!("Welcome! Input desired action here:");
    while let Ok(Some(line)) = BufReader::new(stdin()).lines().next_line().await {
        if line == "exit" {
            break;
        }
        react_to_line(line, &mut plugin)
            .await
            .unwrap_or_else(|e| eprintln!("{e:#?}"));
        println!("\nInput desired action here:");
    }
}

lazy_static! {
    static ref NUMS_PARSER: Regex = Regex::new(r"-?[\d]+(:?\.\d+)?").unwrap();
}

async fn react_to_line(
    line: String,
    plugin: &mut ExamplePluginHandle,
) -> Result<(), Box<dyn StdError>> {
    let nums = NUMS_PARSER
        .find_iter(&line)
        .into_iter()
        .filter_map(|c| Some(c.as_str().parse::<f64>().ok()?))
        .collect::<Vec<_>>();
    if line.starts_with("request_bytes ") {
        let bytes = plugin
            .random_bytes(
                nums.get(0)
                    .ok_or(Error::Generic(
                        "You must specify the amount of bytes desired.".to_string(),
                    ))?
                    .to_string()
                    .parse()?,
            )
            .await?;
        println!("Got {} bytes!", bytes.len());
        return Ok(());
    };
    if let [n1, n2] = nums[..] {
        let result = plugin.float_op(n1, n2).await?;
        println!("Result: {result}");
    } else {
    }
    Ok(())
}
