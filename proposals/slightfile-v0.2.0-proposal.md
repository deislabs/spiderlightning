# `slightfile` v0.2.0 Proposal 
**Made 2022-09-15 by [danbugs](https://github.com/danbugs)**

`slightfile` is the adopted name for a configuration file for `slight` application. Currently, its design imposes some limitations on applications that a user can develop. The purpose of this document is to highlight these limitations and propose a solution. 

## <a name='TableofContents'></a>Table of Contents

<!-- vscode-markdown-toc -->
* [Table of Contents](#TableofContents)
* [Background](#Background)
* [Problem Statement](#ProblemStatement)
* [Proposal](#Proposal)
* [Alternatives](#Alternatives)
	* [Alternative 1](#Alternative1)
	* [Alternative 2](#Alternative2)
	* [Alternative 3](#Alternative3)
	* [Alternative 4](#Alternative4)
	* [Alternative 5](#Alternative5)
* [Additional Details](#AdditionalDetails)
	* [Edge Cases](#EdgeCases)
		* [Duplicate Name](#DuplicateName)

<!-- vscode-markdown-toc-config
	numbering=false
	autoSave=true
	/vscode-markdown-toc-config -->
<!-- /vscode-markdown-toc -->

## <a name='Background'></a>Background

Currently, a common `slightfile` might look like this:
```toml
specversion = "0.1"
secret_store = "configs.azapp"

[[capability]]
name = "kv.azblob"

[[capability]]
name = "mq.azsbus"
```

These capabilities can then be acessed in a user application, like so:
```rs
use anyhow::Result;

use kv::*;
wit_bindgen_rust::import!("../../wit/kv.wit");
wit_error_rs::impl_error!(kv::Error);

use mq::*;
wit_bindgen_rust::import!("../../wit/mq.wit");
wit_error_rs::impl_error!(mq::Error);

fn main() -> Result<()> {
    let kv = Kv::open("the-name-of-your-container")?;
    let mq = Mq::open("the-name-of-your-queue")?;

    // -snip-
}
```

## <a name='ProblemStatement'></a>Problem Statement

The current design has two main restrictions:
(1) we do not allow for multiple-capabilities of the same namespace but different implementor types (e.g., `kv.azblob`, and `kv.filesystem`), and
(2) we do not allow for different capabilities to use different secret stores (i.e., we only have one global secret store).

## <a name='Proposal'></a>Proposal

The changes involve:
- re-naming the `name` field to `resource`,
- re-utilization of the `name` field to create a named resource,
- eliminating the global `secret_store` in favour of being more declarative with configuration variables, and
- creating a map field (i.e., `configs`) for a dynamic number of configs.

```toml
specversion = "0.1"

[[capability]]
resource = "kv.filesystem"
name = "orders"

[[capability]]
resource = "kv.azblob"
name = "customers"
    [capability.configs]
    AZURE_STORAGE_ACCOUNT = "{envvars.AZURE_STORAGE_ACCOUNT}"
    AZURE_STORAGE_KEY = "{envvars.AZURE_STORAGE_KEY}"
```

The example `slightfile` above can be used in code, like so:
```rs
use anyhow::Result;

use kv::*;
wit_bindgen_rust::import!("../../wit/kv.wit");
wit_error_rs::impl_error!(kv::Error);

fn main() -> Result<()> {
    let orders = Kv::open("orders"?; 
    let customers = Kv::open("customers")?;
}
```

It is important to note that there will be a new field added onto every `open` function relating to the name of that capability. Plus, for capabilities that export mulitple resources, one single name will identify them all. For example:

```toml
specversion = "0.1"

[[capability]]
resource = "pubsub.mosquitto"
name = "customer_requests"
    [capability.configs]
    MOSQUITTO_HOST = "{envvars.MOSQUITTO_HOST}"
    MOSQUITTO_PORT = "{envvars.MOSQUITTO_PORT}"
```

In code, this will look like:

```rs
use anyhow::Result;

use kv::*;
wit_bindgen_rust::import!("../../wit/pubsub.wit");
wit_error_rs::impl_error!(pubsub::Error);

fn main() -> Result<()> {
    let publisher = Pub::open("customer_requests")?; 
    let subscriber = Sub::open("customer_requests")?; 
    // both resources exported from the `pubsub.wit` (i.e., `Pub` and `Sub`) 
    // are identified by `customer_requests`.
}
```

## <a name='Alternatives'></a>Alternatives

This section contains all alternatives to the proposal that were explored in the re-design. 

### <a name='Alternative1'></a>Alternative 1

```toml
specversion = "0.1"

[[capability]]
resource = "kv.filesystem"
name = "orders"
configs = "configs.envvars"

[[capability]]
resource = "kv.azblob"
name = "customers"
configs = "configs.azapp"
```

### <a name='Alternative2'></a>Alternative 2

```toml
specversion = "0.1"

[[capability]]
resource = "kv.filesystem"
name = "orders"
configs = "my-configs"

[[capability]]
resource = "kv.azblob"
name = "customers"
configs = "my-configs"

[[capability]]
resource = "kv.azblob"
name = "inventory"
configs = "my-configs"

[[config]]
name = "my-configs"
storage_account = "{envvars.AZURE_STORAGE_ACCOUNT}"
storage_key = "{azapp.AZURE_STORAGE_KEY}"
```

### <a name='Alternative3'></a>Alternative 3

```toml
specversion = "0.1"

# CAPABILITIES
[[capability]]
resource = "kv.azblob"
name = "customers"
configs = ["AZURE_STORAGE_ACCOUNT", "AZURE_STORAGE_KEY"]

[[capability]]
resource = "configs.azapp"
name = "azure-config-store"
configs = ["AZAPPCONFIG_ENDPOINT", "AZAPPCONFIG_KEYID", "AZAPPCONFIG_KEYSECRET"]

[[capability]]
resource = "configs.envvars"
name = "local-config-store"
configs = []

# CONFIGS
[[config]]
name = "AZURE_STORAGE_ACCOUNT"
value = "my-account"

[[config]]
name = "AZURE_STORAGE_KEY"
value = "{configs.azapp.storage_key}"

[[config]]
name = "AZAPPCONFIG_ENDPOINT"
value = "some-endpoint"

[[config]]
name = "AZAPPCONFIG_KEYID"
value = "{configs.envvars.aac_keyid}"

[[config]]
name = "AZAPPCONFIG_KEYSECRET"
value = "{configs.envvars.aac_keysecret}"
```

### <a name='Alternative4'></a>Alternative 4

```toml
specversion = "0.1"
secret_store = "configs.envvars"

[[capability]]
resource = "kv.azblob"
name = "customers"
configs = [{ name = "storage_account", value = "{envvars.AZURE_STORAGE_ACCOUNT}" }, { name = "storage_key", value = "{configs.azapp.storage_pwd}" }]
```

### <a name='Alternative5'></a>Alternative 5

```toml
specversion = "0.1"
secret_store = "configs.envvars"

[[capability]]
resource = "kv.azblob"
name = "customers"
configs = ["envvars.AZURE_STORAGE_ACCOUNT", "azapp.AZURE_STORAGE_ACCOUNT"]
```

## <a name='AdditionalDetails'></a>Additional Details

This section will address some edge cases in this re-design.

### <a name='EdgeCases'></a>Edge Cases

#### <a name='DuplicateName'></a>Duplicate Name

```toml
[[capability]]
resource = "kv.filesystem"
name = "orders"

[[capability]]
resource = "kv.azblob"
name = "orders"
```

If two capabilities have the same name, `slight` should panic.