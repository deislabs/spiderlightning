use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use implementors::SqlImplementor;
use slight_common::{impl_resource, BasicState};
use slight_file::{resource::SqlResource::*, Resource};

mod implementors;
#[cfg(feature = "postgres")]
use implementors::postgres::PostgresImplementor;

use sql::RowItem;
wit_bindgen_wasmtime::export!({paths: ["../../wit/sql.wit"], async: *});
wit_error_rs::impl_error!(sql::SqlError);
wit_error_rs::impl_from!(anyhow::Error, sql::SqlError::UnexpectedError);

#[derive(Clone, Default)]
pub struct Sql {
    implementor: String,
    capability_store: HashMap<String, BasicState>,
}

impl Sql {
    pub fn new(implementor: String, sql: HashMap<String, BasicState>) -> Self {
        Self {
            implementor,
            capability_store: sql,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SqlInner {
    sql_implementor: Arc<dyn SqlImplementor + Send + Sync>,
}

impl SqlInner {
    async fn new(sql_implementor: SqlImplementors, slight_state: &BasicState) -> Self {
        Self {
            sql_implementor: match sql_implementor {
                #[cfg(feature = "postgres")]
                SqlImplementors::Postgres => Arc::new(PostgresImplementor::new(slight_state).await),
            },
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum SqlImplementors {
    #[cfg(feature = "postgres")]
    Postgres,
}

impl From<Resource> for SqlImplementors {
    fn from(s: Resource) -> Self {
        match s {
            #[cfg(feature = "postgres")]
            Resource::Sql(Postgres) => Self::Postgres,
            p => panic!(
                "failed to match provided name (i.e., '{p}') to any known host implementations"
            ),
        }
    }
}

impl_resource!(
    Sql,
    sql::SqlTables<Sql>,
    sql::add_to_linker,
    "sql".to_string()
);

#[derive(Debug)]
pub struct StatementInner {
    query: String,
}

#[async_trait]
impl sql::Sql for Sql {
    type Sql = SqlInner;
    type Statement = StatementInner;

    async fn sql_open(&mut self, name: &str) -> Result<Self::Sql, sql::SqlError> {
        let state = if let Some(r) = self.capability_store.get(name) {
            r.clone()
        } else if let Some(r) = self.capability_store.get(&self.implementor) {
            r.clone()
        } else {
            panic!(
                "could not find capability under name '{}' for implementor '{}'",
                name, &self.implementor
            );
        };

        tracing::log::info!("Opening implementor {}", &state.implementor);

        let inner = Self::Sql::new(state.implementor.into(), &state).await;

        Ok(inner)
    }
    async fn sql_query(
        &mut self,
        self_: &Self::Sql,
        statement: &Self::Statement,
    ) -> Result<Vec<RowItem>, sql::SqlError> {
        Ok(self_.sql_implementor.query(&statement.query).await?)
    }
    async fn sql_exec(
        &mut self,
        self_: &Self::Sql,
        statement: &Self::Statement,
    ) -> Result<(), sql::SqlError> {
        Ok(self_.sql_implementor.exec(&statement.query).await?)
    }

    async fn statement_prepare(&mut self, query: &str, params: Vec<&str>) -> Self::Statement {
        let mut prepared_query = String::from(query);
        let mut param_index = 0;
        while let Some(start_index) = prepared_query.find('?') {
            let end_index = start_index + 1;
            if params.len() <= param_index {
                panic!("Not enough parameters provided for the query");
            }
            let param = &params[param_index];

            let mut quoted_param = "'".to_string();
            for ch in param.chars() {
                quoted_param.push(match ch {
                    '\'' => '\'',
                    '\\' => '\\',
                    _ => ch,
                });
            }
            quoted_param.push('\'');
            prepared_query.replace_range(start_index..end_index, &quoted_param);
            param_index += 1;
        }
        if params.len() != param_index {
            panic!("Too many parameters provided for the query");
        }

        StatementInner {
            query: prepared_query,
        }
    }
}
