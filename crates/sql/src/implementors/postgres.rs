use anyhow::Result;
use async_trait::async_trait;
use postgres::{Client, NoTls};
use slight_common::BasicState;
use slight_runtime_configs::get_from_state;
use tokio::task::block_in_place;
use chrono::{NaiveDate, NaiveTime, NaiveDateTime};

use crate::sql::DataType;

use super::SqlImplementor;

#[derive(Clone)]
pub struct PostgresImplementor {
    connection_url: String,
}

impl PostgresImplementor {
    pub async fn new(slight_state: &BasicState) -> Self {
        Self {
            connection_url: get_from_state("POSTGRES_CONNECTION_URL", slight_state)
                .await
                .unwrap(),
        }
    }
}

impl std::fmt::Debug for PostgresImplementor {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "PostgresImplementor")
    }
}

#[async_trait]
impl SqlImplementor for PostgresImplementor {
    async fn query(&self, query: &str) -> Result<Vec<crate::sql::RowItem>> {
        block_in_place(|| {
            let mut client = Client::connect(&self.connection_url, NoTls)?;
            let mut row_result = Vec::new();
            for row in client.query(query, &[])? {
                for (i, c) in row.columns().iter().enumerate() {
                    let value = match c.type_().name() {
                        "integer" => {
                            let v: i32 = row.get(i);
                            DataType::Int32(v)
                        }
                        "bigint" => {
                            let v: i64 = row.get(i);
                            DataType::Int64(v)
                        }
                        "smallint" => {
                            let v: i16 = row.get(i);
                            DataType::Int32(v as i32)
                        }
                        "real" => {
                            let v: f32 = row.get(i);
                            DataType::Float(v as f64)
                        }
                        "double precision" => {
                            let v: f64 = row.get(i);
                            DataType::Double(v)
                        }
                        "text" => {
                            let v: String = row.get(i);
                            DataType::Str(v)
                        }
                        "boolean" => {
                            let v: bool = row.get(i);
                            DataType::Boolean(v)
                        }
                        "date" => {
                            let v: String = row.get(i);
                            let parsed = NaiveDate::parse_from_str(&v, "%Y-%m-%d").unwrap();
                            DataType::Date(parsed.to_string())
                        }
                        "time" => {
                            let v: String = row.get(i);
                            let parsed = NaiveTime::parse_from_str(&v, "%H:%M:%S").unwrap();
                            DataType::Time(parsed.to_string())
                        }
                        "timestamp" => {
                            let v: String = row.get(i);
                            let parsed = NaiveDateTime::parse_from_str(&v, "%Y-%m-%d %H:%M:%S").unwrap();
                            DataType::Timestamp(parsed.to_string())
                        }
                        "bytea" => {
                            let v: Vec<u8> = row.get(i);
                            DataType::Binary(v)
                        }
                        _ => DataType::Null,
                    };

                    row_result.push(crate::sql::RowItem {
                        field_name: c.name().to_string(),
                        value,
                    });
                }
            }
            Ok(row_result)
        })
    }

    async fn exec(&self, query: &str) -> Result<()> {
        block_in_place(|| {
            let mut client = Client::connect(&self.connection_url, NoTls)?;
            client.execute(&query.to_string(), &[])?;
            Ok(())
        })
    }
}
