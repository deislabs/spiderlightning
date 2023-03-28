use anyhow::{bail, Result};
use async_trait::async_trait;
use aws_config::{from_env, meta::region::RegionProviderChain};
use aws_sdk_dynamodb::model::{AttributeValue, Select};
use aws_sdk_dynamodb::Client;

use slight_common::BasicState;
use slight_runtime_configs::get_from_state;
use tracing::log;

use super::KeyvalueImplementor;

/// This is the underlying struct behind the "AWS DynamoDB" variant of the `KeyvalueImplementor` enum.
///
/// It provides a properties that pertains solely to the AWS DynamoDB implementation
/// of this capability:
///    - `client`, and
///   - `table_name`,
#[derive(Debug, Clone)]
pub struct AwsDynamoDbImplementor {
    client: Client,
    table_name: String,
}

impl AwsDynamoDbImplementor {
    /// Creates a new `AwsDynamoDbImplementor` instance.
    ///
    /// It uses the `aws_config::from_env()` for AWS Configuration.
    /// It will access the AWS Configuration environment variables:
    ///   - `AWS_ACCESS_KEY_ID`, and
    ///   - `AWS_SECRET_ACCESS_KEY`, and
    ///   - `AWS_REGION`, or `AWS_DEFAULT_REGION`
    ///
    /// In order to use the AWS DyanmoDB implementor, you must have a DynamoDB table
    /// with a primary key named `key`.
    ///
    /// The layout of the DynamoDB table is as follows:
    /// ```text
    /// {
    ///   "key": {
    ///       "S": <key>
    ///   },
    ///   "value": {
    ///       "S": <value>
    ///   }
    /// }
    /// ```
    pub async fn new(slight_state: &BasicState, name: &str) -> Self {
        let access_id = get_from_state("AWS_ACCESS_KEY_ID", slight_state).await.unwrap();
        std::env::set_var("AWS_ACCESS_KEY_ID", access_id);
        
        let access_key = get_from_state("AWS_SECRET_ACCESS_KEY", slight_state).await.unwrap();
        std::env::set_var("AWS_SECRET_ACCESS_KEY", access_key);
        
        let region = get_from_state("AWS_REGION", slight_state).await;
        let default_region = get_from_state("AWS_DEFAULT_REGION", slight_state).await;
        if region.is_err() && default_region.is_err() {
            panic!("AWS_REGION or AWS_DEFAULT_REGION must be set");
        } else if region.is_err() {
            std::env::set_var("AWS_DEFAULT_REGION", default_region.unwrap());
        } else {
            std::env::set_var("AWS_REGION", region.unwrap());
        }
        
        let region = RegionProviderChain::default_provider();
        let config = from_env().region(region).load().await;
        let client = Client::new(&config);
        let table_name = name.into();
        log::info!(
            "Creating a new AWS DynamoDB resource with table name: {}",
            name
        );
        Self { client, table_name }
    }
}

#[async_trait]
impl KeyvalueImplementor for AwsDynamoDbImplementor {
    async fn get(&self, key: &str) -> Result<Vec<u8>> {
        let key_attribute = AttributeValue::S(key.into());
        log::info!("Getting value from key: {}", key);
        let res = self
            .client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression("#key = :value".to_string())
            .expression_attribute_names("#key".to_string(), "key".to_string())
            .expression_attribute_values(":value".to_string(), key_attribute)
            .select(Select::AllAttributes)
            .send()
            .await?;
        match res.items.unwrap_or_default().pop() {
            Some(item) => {
                let value = item.get("value").unwrap();
                let value = value.as_s().unwrap();
                Ok(value.as_bytes().to_vec())
            }
            None => bail!("no value found for key: {}", key),
        }
    }

    async fn set(&self, key: &str, value: &[u8]) -> Result<()> {
        let key_attribute = AttributeValue::S(key.into());
        let value = AttributeValue::S(
            String::from_utf8(value.to_vec()).expect("failed to convert value to String"),
        );
        log::info!("Setting key value pair: ({}, {:#?})", key, value);

        self.client
            .put_item()
            .table_name(&self.table_name)
            .item("key", key_attribute)
            .item("value", value)
            .send()
            .await?;
        Ok(())
    }

    async fn keys(&self) -> Result<Vec<String>> {
        let res = self
            .client
            .scan()
            .table_name(&self.table_name)
            .select(Select::AllAttributes)
            .send()
            .await?;
        let items = res.items.unwrap_or_default();
        let keys = items
            .iter()
            .map(|item| item.get("key").unwrap().as_s().unwrap().to_string())
            .collect();
        Ok(keys)
    }

    /// FIXME: should delete return a success if it is a noop
    /// or should it return an error if the key is not found?
    async fn delete(&self, key: &str) -> Result<()> {
        let key_attribute = AttributeValue::S(key.into());
        log::info!("Deleting key: {}", key);
        self.client
            .delete_item()
            .table_name(&self.table_name)
            .key("key", key_attribute)
            .send()
            .await?;
        Ok(())
    }
}
