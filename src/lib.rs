use std::io;

pub use async_graphql;

use async_graphql::EmptyMutation;
use async_graphql::EmptySubscription;
use async_graphql::Object;
use async_graphql::Schema;
use async_graphql::SimpleObject;

use sqlx::postgres::PgPool;

#[derive(Debug, SimpleObject)]
pub struct PgTable {
    pub table_catalog: Option<String>,
    pub table_schema: Option<String>,
    pub table_name: Option<String>,
    pub table_type: Option<String>,
    pub self_referencing_column_name: Option<String>,
    pub reference_generation: Option<String>,
    pub user_defined_type_catalog: Option<String>,
    pub user_defined_type_schema: Option<String>,
    pub user_defined_type_name: Option<String>,
    pub is_insertable_into: Option<String>,
    pub is_typed: Option<String>,
    pub commit_action: Option<String>,
}

pub async fn get_pg_table_info_all(
    p: &PgPool,
    schema_pattern: &str,
    name_pattern: &str,
) -> Result<Vec<PgTable>, io::Error> {
    sqlx::query_as!(
        PgTable,
        r#"(
            SELECT
                table_catalog::TEXT,
                table_schema::TEXT,
                table_name::TEXT,
                table_type::TEXT,
                self_referencing_column_name::TEXT,
                reference_generation::TEXT,
                user_defined_type_catalog::TEXT,
                user_defined_type_schema::TEXT,
                user_defined_type_name::TEXT,
                is_insertable_into::TEXT,
                is_typed::TEXT,
                commit_action::TEXT
            FROM information_schema.tables
            WHERE
              table_schema LIKE $1::TEXT
              AND table_name LIKE $2::TEXT
        )"#,
        schema_pattern,
        name_pattern,
    )
    .fetch_all(p)
    .await
    .map_err(io::Error::other)
}

pub struct Query {
    pool: PgPool,
}

fn default_schema_pattern() -> String {
    "%".to_string()
}

fn default_name_pattern() -> String {
    "%".to_string()
}

#[Object]
impl Query {
    async fn tables(
        &self,

        #[graphql(
            desc = "Filter tables by schema name. Use '%' for wildcard matching.",
            default_with = "default_schema_pattern()"
        )]
        schema_pattern: String,

        #[graphql(
            desc = "Filter tables by table name. Use '%' for wildcard matching.",
            default_with = "default_name_pattern()"
        )]
        name_pattern: String,
    ) -> Result<Vec<PgTable>, io::Error> {
        let p: &PgPool = &self.pool;
        get_pg_table_info_all(p, &schema_pattern, &name_pattern).await
    }
}

pub type TablesSchema = Schema<Query, EmptyMutation, EmptySubscription>;

pub async fn pool_new(conn_str: &str) -> Result<PgPool, io::Error> {
    PgPool::connect(conn_str).await.map_err(io::Error::other)
}

pub async fn query_new(conn_str: &str) -> Result<Query, io::Error> {
    let pool = pool_new(conn_str).await?;
    Ok(Query { pool })
}
