TODO:
-- rewording
-- logical resource configuration
-- errors

# Design Principles

The below is a collection of principles that guide the overall interface development.

## Coordinated Cancelation Support
WASI-Cloud must support coordinated cancelation across all data plane functions. The support follows golang style
context. However the context in wasi is a simple `u64` wide int with the least significant bit set when canceled.The context does not carry any other information. The remaining bits are reserved for possible future use.

Timeouts, Cancel-After, and hooking the token to language intrinsic coordinated cancelation is left for language specific implementation.

The cancelation token is assumed to be respected in the following contexts:
1. When WASM app calls into the host. If the token was canceled the host must return with "canceled by caller" style error.
2. When WASI-Cloud host calls back into the WASM app and passes a token. Token canceled by host must be respected by WASM app returning the same "canceled by caller" error.


## Errors

!TBD!

## Resource Descriptors
Capabilities are used as resource. As an example queuing capability is interacted with via connecting to a queue (resource), sending message to a queue and then finally closing the queue.

Resources offered by WASI-Cloud host are represented by a `u64` integer type. Each resource instance (e.g., key/value store or a queue) is represented by a unique value created by the host. The resource descriptor closely resembles Linux's file descriptor `FD` or Windows's `Handle`. The following rules apply:

1. The value of the descriptor is opaque. It is left for WASI-Cloud host to create and manage.
2. All WASI-Cloud hosts offer `Close(descriptor)` function for WASM apps to signify that resource is no longer needed. Allowing WASI-Cloud host to perform whatever needed to cleanup the resources (e.g., close network listeners, clear cache etc).
3. Unlike Linux, those descriptors are not be shared cross processes.
4. Standard error exists for error stemming from invalid descriptors, descriptor closed, etc.. 


## Logical v Physical Resource and Resource Configuration
Resources represents the first step for a WASM application to interact with capabilities offered by the WASI-Cloud host. A typical WASM program looks like:

```golang
// xxx == resource type
resourceDescriptor := GetXXX() // error management is omitted.
/* use resource */
close(resourceDescriptor)
```

The above entails:
1.  Resource _was_ configured on the WASI-Cloud host. This prior action can be done via configuration to the WASI-Cloud host using whatever means the WASI-Cloud host implementation dictates or using the WASI-Cloud configuration interface.
2. Resource is physically mapped to the provider representation for example
```
// The below means that host is configured with a queue named order that maps to actual implementation of queue
// infrastructure. e.g., a service bus queue or kafka queue.
queueDescriptor := GetQueue("USWest-Orders")
..
```

Similarly WASM application can choose to interact with logical queues. Again, the assumption here is the WASI-Cloud host is pre-configured. For example:

```golang
queueDescriptor := GetLogicalQueue("Orders")
```

`Orders` queue can be mapped to anything including and not limited to an aggregate of queues. Please note the above does not dedicate how the WASI-Cloud host was configured to create the mapping needed to support `orders` queue.

### Resource Configuration
WASM applications running on WASI-Cloud hosts can alternatively choose to configure the WASI-Cloud host capabilities before using them. As an example:

```golang
// capability configuration is just a map of properties 
capConfig := map[string]string{
 "type" : "AZURE_SERVICE_BUS",
 "name" : "insurance_policies",
 /* free standing key/value form eg:*/
  "connectionString" : "....", 
}
// TODO: Configuring logical queue
// Then it can be used to configure the host from the WASM app
// note cap configuration  created by WASM app is also a resource
capDescriptor = ConfigureCapability(capConfig)
// now WASM app can use it 
queueDescriptor := GetQueue("insurance_policies")
``` 

## Note on OOAD Interfaces 
The interfaces must be design without dependencies on any OOAD specific features such as virtual methods, overloads among others. Instead interfaces must be basic function calls (POSIX style) that operate on basic data types. Languages implementing WASI-Cloud may choose to have a layer on top of WASI-Cloud. 
