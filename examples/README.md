# working with the samples

## preparing an Azure environment

All samples with `configs.azapp`, `kv.azblob` or `mq.azsbus` capabilities need a defined set of resources. With shell commands below this required resources can be created and configuration values are put into an Azure App Configuration.

> This configuration currently only supports the examples on the local filesystem and with **Azure** resources. Configuration of **AWS** resources and **etcd** are not yet covered.

```shell
SUBSCRIPTION_ID={your-azure-subscription}
LOCATION={your-desired-azure-location}
RESOURCE_GROUP={your-desired-azure-resource-group-name}
APPCONFIG_NAME={your-desired-azure-app-configuration-name}
STORAGE_ACCOUNT={your-desired-azure-storage-account-name}
SERVICEBUS_NAMESPACE={your-desired-azure-servicebus-namespace}
QUEUENAME=wasi-cloud-queue
CONTAINERNAME1=my-container
CONTAINERNAME2=my-container2

az account set -s $SUBSCRIPTION_ID

az group create -n $RESOURCE_GROUP -l $LOCATION

az configure --defaults location=$LOCATION group=$RESOURCE_GROUP

az appconfig create -n $APPCONFIG_NAME

az servicebus namespace create -n $SERVICEBUS_NAMESPACE
az servicebus queue create --namespace-name $SERVICEBUS_NAMESPACE --name $QUEUENAME

SERVICEBUS_AUTHRULE=`az servicebus namespace authorization-rule list --namespace $SERVICEBUS_NAMESPACE --query '[0].name' -o tsv`

az storage account create -n $STORAGE_ACCOUNT
az storage container create --account-name $STORAGE_ACCOUNT -n $CONTAINERNAME1
az storage container create --account-name $STORAGE_ACCOUNT -n $CONTAINERNAME2

az appconfig kv set -n $APPCONFIG_NAME --key AZURE_STORAGE_ACCOUNT --value $STORAGE_ACCOUNT -y
az appconfig kv set -n $APPCONFIG_NAME --key AZURE_STORAGE_KEY --value `az storage account keys list --account-name $STORAGE_ACCOUNT --query [0].value -o tsv` -y
az appconfig kv set -n $APPCONFIG_NAME --key AZURE_SERVICE_BUS_NAMESPACE --value $SERVICEBUS_NAMESPACE -y
az appconfig kv set -n $APPCONFIG_NAME --key AZURE_POLICY_NAME --value $SERVICEBUS_AUTHRULE -y
az appconfig kv set -n $APPCONFIG_NAME --key AZURE_POLICY_KEY --value `az servicebus namespace authorization-rule keys list --namespace $SERVICEBUS_NAMESPACE --name $SERVICEBUS_AUTHRULE --query primaryKey -o tsv` -y

APPCONFIG_CONNECTION=$(az appconfig credential list -n $APPCONFIG_NAME --query "[?name=='Primary'].connectionString" -o tsv)
export AZAPPCONFIG_ENDPOINT=$(echo $APPCONFIG_CONNECTION | cut -d';' -f1 | cut -d'=' -f2)
export AZAPPCONFIG_KEYID=$(echo $APPCONFIG_CONNECTION | cut -d';' -f2 | cut -d'=' -f2)
export AZAPPCONFIG_KEYSECRET=$(echo $APPCONFIG_CONNECTION | cut -d';' -f3 | cut -c8-60)
```

When configuration values are set, the Rust set of examples can be build and executed:

```shell
rustup target add wasm32-wasi
make install-deps
make build
sudo make install-slight
make build-rust
make run-rust
```
