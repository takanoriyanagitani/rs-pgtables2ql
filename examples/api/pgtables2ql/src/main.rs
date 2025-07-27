use std::process::ExitCode;

use std::io;

use tokio::net::TcpListener;

use axum::Router;

use rs_pgtables2ql::async_graphql;

use async_graphql::EmptyMutation;
use async_graphql::EmptySubscription;
use async_graphql::Schema;

use async_graphql_axum::GraphQLRequest;
use async_graphql_axum::GraphQLResponse;

use rs_pgtables2ql::Query;
use rs_pgtables2ql::TablesSchema;
use rs_pgtables2ql::query_new;

async fn req2res(s: &TablesSchema, req: GraphQLRequest) -> GraphQLResponse {
    s.execute(req.into_inner()).await.into()
}

fn get_conn_str() -> Result<String, io::Error> {
    std::env::var("DATABASE_URL").map_err(io::Error::other)
}

fn get_listen_addr() -> String {
    std::env::var("LISTEN_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_string())
}

async fn sub() -> Result<(), io::Error> {
    let conn_str: String = get_conn_str()?;
    let addr_port: String = get_listen_addr();

    let query: Query = query_new(&conn_str).await?;
    let schema: TablesSchema = Schema::build(query, EmptyMutation, EmptySubscription).finish();

    let sdl: String = schema.sdl();
    std::fs::write("pg-tables.graphql", sdl.as_bytes())?;

    let app = Router::new().route(
        "/",
        axum::routing::post(|req: GraphQLRequest| async move { req2res(&schema, req).await }),
    );

    let listener = TcpListener::bind(addr_port).await?;

    axum::serve(listener, app).await
}

#[tokio::main]
async fn main() -> ExitCode {
    if let Err(e) = sub().await {
        eprintln!("Error running server: {e}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
