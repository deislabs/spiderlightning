# `slightfile` Re-Design

`slightfile` is the adopted name for a configuration file for `slight` application. Currently, its design imposes some limitations on applications that a user can develop. The purpose of this document is to highlight these limitations and propose a solution. 

## <a name='TableofContents'></a>Table of Contents

<!-- vscode-markdown-toc -->
* [Table of Contents](#TableofContents)
* [Background](#Background)
* [Problem Statement](#ProblemStatement)
* [Proposal](#Proposal)
	* [Option 1](#Option1)
	* [Option 2](#Option2)
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

There are a couple options to solve the highlited problem.

### <a name='Option1'></a>Option 1

```toml
specversion = "0.1"
secret_store = "configs.envvars"


[[capability]]
resource = "kv.filesystem"
name = "orders"

[[capability]]
resource = "kv.azblob"
name = "customers"
configs = "configs.azapp"
```

The changes are:
- there is no longer the need to use a globally defined `secret_store` — now, one has the option to individually attach to each capability their own independent config referred to by name, and, if none is provided, `slight` can just fallback to the globally defined one,
- renaming of the previously called `name` field to `resource`, and
- we now have a new `name` field, which is the name a capability can be referred to in code.

The example `slightfile` above can be used in code, like so:
```rs
use anyhow::Result;

use kv::*;
wit_bindgen_rust::import!("../../wit/kv.wit");
wit_error_rs::impl_error!(kv::Error);

fn main() -> Result<()> {
    let orders = Kv::open("orders", "orders-folder")?; 
    let customers = Kv::open("customers", "customers-container")?;
}
```

It is important to note that there will be a new field added onto every `open` function relating to the name of that capability. Plus, for capabilities that export mulitple resources, one single name will identify them all. For example:

```toml
specversion = "0.1"
secret_store = "configs.envvars"


[[capability]]
resource = "pubsub.mosquitto"
name = "customer_requests"
configs = "configs.azapp"
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

**Pros:**
- easy to implement.
**Cons:**
- does not improve the fact configurations are not declarative. That is, currently, as a user, it is hard to figure out what kind of configurations a service requires without looking at the source code or through trial and error (i.e., via error messsages).

### <a name='Option2'></a>Option 2

```toml
specversion = "0.1"
secret_store = "configs.envvars"


[[capability]]
resource = "kv.filesystem"
name = "orders"

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

The changes are:
- there is no longer the need to use a globally defined `secret_store` — now, one has the option to individually attach to each capability their own independent config referred to by name, and, if none is provided, `slight` can just fallback to the globally defined one,
- rename of the previously called `name` field to `resource`,
- we now have a new `name` field, which is the name a capability can be referred to in code, and
- a new `[[config]]` section that defines the name it can be referred to by capabilities and some configuration key-value pairs namespaced by config type (i.e., `envvars`, or `azapp`).

**Pros:**
- configurations can be explicitly shared by multiple capabilities.
- configurations become more declarative.

**Cons:**
- adding implementors that require new configurations will require modifying the `slightfile` spec (e.g., say we add a new sql implementor that requires a ` connection_string` key-value config, to be able to still serialize the `slightfile` with this new field, wewill have to modify the `slightfile` `struct` to contain an `Option<String>` for `connection_string`).


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