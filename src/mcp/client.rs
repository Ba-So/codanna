//! MCP client for connecting to the code intelligence server

use anyhow::{Result, anyhow};
use serde_json::Value;
use std::path::PathBuf;

pub struct CodeIntelligenceClient;

impl CodeIntelligenceClient {
    /// Connect to MCP server and test it (thin client - no index loading)
    pub async fn test_server(
        server_binary: PathBuf,
        config_path: Option<PathBuf>,
        tool: Option<String>,
        args: Option<String>,
        delay_before_tool_secs: Option<u64>,
    ) -> Result<()> {
        use rmcp::{
            model::{CallToolRequestParam, JsonObject},
            service::ServiceExt,
            transport::{ConfigureCommandExt, TokioChildProcess},
        };
        use tokio::process::Command;
        use tokio::time::{Duration, sleep};

        println!("Starting MCP server process...");

        let client = ()
            .serve(TokioChildProcess::new(
                Command::new(&server_binary).configure(|cmd| {
                    if let Some(cfg) = &config_path {
                        cmd.arg("--config");
                        cmd.arg(cfg);
                    }

                    cmd.arg("serve");
                }),
            )?)
            .await?;

        // Get server info
        let server_info = client.peer_info();
        println!("Connected to server: {server_info:#?}");

        // List tools
        println!("\nListing available tools...");
        let tools = client.list_tools(Default::default()).await?;

        for tool in &tools.tools {
            println!(
                "  - {}: {}",
                tool.name,
                tool.description.as_deref().unwrap_or("No description")
            );
        }

        // Always call get_index_info first to verify semantic availability
        println!("\nCalling get_index_info tool...");
        let get_info_result = client
            .call_tool(CallToolRequestParam {
                name: "get_index_info".into(),
                arguments: None,
            })
            .await?;
        Self::print_tool_output(&get_info_result);

        // Optionally call a specific tool supplied by the user
        if let Some(tool_name) = tool {
            if let Some(delay) = delay_before_tool_secs {
                if delay > 0 {
                    println!("\nWaiting {delay} seconds before calling '{tool_name}'...");
                    sleep(Duration::from_secs(delay)).await;
                }
            }

            println!("\nCalling tool '{tool_name}'...");

            let parsed_args: Option<JsonObject> = if let Some(raw) = args.as_ref() {
                let value: Value = serde_json::from_str(raw)
                    .map_err(|e| anyhow!("Failed to parse --args as JSON object: {e}"))?;

                match value {
                    Value::Object(map) => Some(map),
                    _ => {
                        return Err(anyhow!(
                            "Tool arguments must be a JSON object (e.g. {{\"query\":\"test\"}})"
                        ));
                    }
                }
            } else {
                None
            };

            let tool_result = client
                .call_tool(CallToolRequestParam {
                    name: tool_name.into(),
                    arguments: parsed_args,
                })
                .await?;
            Self::print_tool_output(&tool_result);
        }

        // Shutdown
        println!("\nShutting down...");
        client.cancel().await?;

        Ok(())
    }

    fn print_tool_output(result: &rmcp::model::CallToolResult) {
        println!("Result:");
        for annotated_content in &result.content {
            match &**annotated_content {
                rmcp::model::RawContent::Text(text) => println!("{}", text.text),
                _ => println!("(Non-text content)"),
            }
        }

        if result.is_error.unwrap_or(false) {
            println!("Tool returned an error status");
        }
    }
}
