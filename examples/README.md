# working with the samples

## preparing an Azure environment

All samples with `kv.azblob` or `mq.azsbus` capabilities need a defined set of resources. With a shell script as below this required resources can be created and `export AZURE_xxx="value"` statements are printed out, which can be copy / pasted from a system with Azure CLI to the system (or DevContainer or GitHub Codespace) which is actually running the samples. In case this is the same system, echoing can be removed and the environment variables can be set directly.

```shell
SUBSCRIPTION_ID={your-azure-subscription}
LOCATION={your-desired-azure-location}
RESOURCE_GROUP={your-desired-azure-resource-group-name}
STORAGE_ACCOUNT={your-desired-azure-storage-account-name}
SERVICEBUS_NAMESPACE={your-desired-azure-servicebus-namespace}
QUEUENAME=wasi-cloud-queue
CONTAINERNAME1=my-container
CONTAINERNAME2=my-container2

az account set -s $SUBSCRIPTION_ID

az group create -n $RESOURCE_GROUP -l $LOCATION

az configure --defaults location=$LOCATION group=$RESOURCE_GROUP

az servicebus queue create --namespace-name $SERVICEBUS_NAMESPACE --name $QUEUENAME

az storage container create --account-name $STORAGE_ACCOUNT -n $CONTAINERNAME1
az storage container create --account-name $STORAGE_ACCOUNT -n $CONTAINERNAME2

echo "export AZURE_STORAGE_ACCOUNT=\"$STORAGE_ACCOUNT\""

echo "export AZURE_STORAGE_KEY=`az storage account keys list --account-name $STORAGE_ACCOUNT --query [0].value`"

echo "export AZURE_SERVICE_BUS_NAMESPACE=\"$SERVICEBUS_NAMESPACE\""

SERVICEBUS_AUTHRULE=`az servicebus namespace authorization-rule list --namespace $SERVICEBUS_NAMESPACE --query '[0].name' -o tsv`

echo "export AZURE_POLICY_NAME=\"$SERVICEBUS_AUTHRULE\""

echo "export AZURE_POLICY_KEY=`az servicebus namespace authorization-rule keys list --namespace $SERVICEBUS_NAMESPACE --name $SERVICEBUS_AUTHRULE --query primaryKey`"
```

When environment variables are set, the Rust set of examples can be build and executed:

```shell
$ make install-deps # installs the WASI-SDK
$ make build # builds SpiderLightning/Slight
$ sudo make install-slight # installs so that it can be used by samples
$ make build-rust # build Rust samples
$ make run-rust # run Rust samples
```
