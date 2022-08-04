use anyhow::{bail, Result};
use aws_sdk_dynamodb::model::{AttributeValue, Select};
use aws_sdk_dynamodb::{Client};
use futures::executor::block_on;

use tracing::log;

/// This is the underlying struct behind the "AWS DynamoDB" variant of the `KvImplementor` enum.
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
    /// It uses the `aws_config::load_from_env()` for AWS Configuration.
    /// It will access the AWS Configuration environment variables:
    ///   - `AWS_ACCESS_KEY_ID`, and
    ///   - `AWS_SECRET_ACCESS_KEY`, and
    ///   - `AWS_REGION`.
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
    pub fn new(name: &str) -> Self {
        let shared_config = block_on(aws_config::load_from_env());
        let client = Client::new(&shared_config);
        let table_name = name.into();
        log::info!(
            "Creating a new AWS DynamoDB resource with table name: {}",
            name
        );
        Self { client, table_name }
    }

    pub fn get(&self, key: &str) -> Result<Vec<u8>> {
        let key_attribute = AttributeValue::S(key.into());
        log::info!("Getting value from key: {}", key);
        let res = block_on(
            self.client
                .query()
                .table_name(&self.table_name)
                .key_condition_expression("#key = :value".to_string())
                .expression_attribute_names("#key".to_string(), "key".to_string())
                .expression_attribute_values(":value".to_string(), key_attribute)
                .select(Select::AllAttributes)
                .send(),
        )?;
        match res.items.unwrap_or_default().pop() {
            Some(item) => {
                let value = item.get("value").unwrap();
                let value = value.as_s().unwrap();
                Ok(value.as_bytes().to_vec())
            }
            None => bail!("no value found for key: {}", key),
        }
    }

    pub fn set(&self, key: &str, value: &[u8]) -> Result<()> {
        let key_attribute = AttributeValue::S(key.into());
        let value = AttributeValue::S(
            String::from_utf8(value.to_vec()).expect("failed to convert value to String"),
        );
        log::info!("Setting key value pair: ({}, {:#?})", key, value);
        block_on(
            self.client
                .put_item()
                .table_name(&self.table_name)
                .item("key", key_attribute)
                .item("value", value)
                .send(),
        )?;
        Ok(())
    }

    /// FIXME: should delete return a success if it is a noop
    /// or should it return an error if the key is not found?
    pub fn delete(&self, key: &str) -> Result<()> {
        let key_attribute = AttributeValue::S(key.into());
        log::info!("Deleting key: {}", key);
        block_on(
            self.client
                .delete_item()
                .table_name(&self.table_name)
                .key("key", key_attribute)
                .send(),
        )?;
        Ok(())
    }
}
