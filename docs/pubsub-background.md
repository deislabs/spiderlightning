# Pub/Sub Study

This document describes some background research that inspired the design of the Pub/Sub capability.

## Definition

The Publisher/Subscriber (Pub/Sub) pattern is a messaging system pattern that allows services to communicate asynchronously with each other.

### Pub/Sub vs Request Response (RPC)
It differs from the traditional request/response pattern in that the sender of a message (aka the publisher/producer) does not expect a response from the receiver (aka the subscriber/consumer) immediately. Instead, the sender sends a message to a message broker, which then delivers the message to the receiver. In this way, the sender and receiver are decoupled from each other. Both the sender and the receiver are clients to the message broker.


### Message Broker

> Technically, the Pub/Sub pattern does not require a message broker. Unix pipe, UDP protocol, and ZeroMQ are examples of message brokers that do not require a message broker. However, we will focus on the message broker implementation in this document.

The message broker is responsible for a few things:

- Persisting the message until the receiver consumes it, except for the case where the queue is implemented in memory.
- Delivering the message to the receivers.
- Fault tolerance. If the receiver is down, the message broker should be able to retry the delivery until the receiver is back online.

### Quality of Service

The message broker is expected each or all of the following qualities of service (QoS):

- At most once or best-effort delivery: The message is delivered at most once. The message may be lost, but it will never be delivered twice.
- At least once delivery: The message is delivered at least once. The message may be delivered multiple times, but it will never be lost.
- Effectively once delivery: The message is delivered effectively once. The message will never be lost or delivered twice.

### Multiple Receivers

The message broker should be able to deliver the same message to multiple receivers. There are two ways to do this:

- Topic subscription (aka fan-out): The message broker delivers the message to all receivers that are subscribed to the topic.

- Queue (aka load-balancing): The message broker delivers the message to one of the receivers that are subscribed to the queue. The message broker should be able to load balance the message delivery to the receivers.

### Push vs. Pull Message Delivery

The message broker can deliver the message to the receiver in two ways:

- Push: The message broker calls the `on_message()` function implemented by the receiver component.

- Pull: The receiver pulls the message from the message broker by invoking the `receive()` function.
