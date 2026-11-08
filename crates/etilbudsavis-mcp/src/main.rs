//! An MCP server over eTilbudsavis, speaking JSON-RPC on stdin and stdout.

use std::process::ExitCode;

use clap::Parser as _;
use etilbudsavis_mcp::cli::{self, Args, StartupError};
use etilbudsavis_mcp::server::EtibudsavisServer;
use rmcp::ServiceExt as _;
use rmcp::transport::stdio;

#[tokio::main]
async fn main() -> ExitCode {
    let args = Args::parse();
    cli::init_tracing();

    match run(args).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            report(&error);
            ExitCode::from(2)
        }
    }
}

async fn run(args: Args) -> Result<(), StartupError> {
    let mode = args.mode()?;

    if args.list_tools {
        let tools = EtibudsavisServer::router(mode).list_all();
        println!("{}", serde_json::to_string_pretty(&tools)?);
        return Ok(());
    }

    let client = args.client().await?;

    if args.check {
        let state = client
            .auth_state()
            .await
            .map_err(|error| StartupError::Rejected(Box::new(error)))?;
        match state.person {
            Some(person) => println!(
                "signed in as {} ({}{})",
                person.email.as_deref().unwrap_or("anonymous"),
                person.id,
                if person.is_anonymous {
                    ", anonymous"
                } else {
                    ""
                }
            ),
            None => println!("no account token; public tools still work"),
        }
        return Ok(());
    }

    let server = EtibudsavisServer::new(client, args.limits(), mode);
    match server.serve(stdio()).await {
        Ok(service) => {
            if let Err(error) = service.waiting().await {
                tracing::error!(%error, "the connection ended badly");
            }
        }
        Err(error) => tracing::error!(%error, "could not start serving"),
    }

    Ok(())
}

fn report(error: &StartupError) {
    eprintln!("etilbudsavis-mcp: {error}");
    let mut source = std::error::Error::source(error);
    while let Some(cause) = source {
        eprintln!("  caused by: {cause}");
        source = cause.source();
    }
}
