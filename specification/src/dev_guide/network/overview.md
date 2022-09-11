# `pluto-network`

All networking APIs are defined in this library crate.
Other binaries and libraries will refer to `pluto-network` for communication.

## `Client`

`Client` is the heart of the system; it is responsible for sending
messages between nodes on the network via MQTT.

When sending a request, the `Client` uses a handler to await for incoming
messages on a given topic, and will automatically parse the message into
the correct protobuf response structure, defined by the `Request` trait implementation.
This includes encrypted messages, where the type system can expect an encrypted
message of a given type.

## Messages

Messages come in two variants: encrypted and unencrypted.
These are reflected in the types `Message<M>` and `EncryptedMessage<M>` respectively.

Any protobuf message can be encrypted via the `Encrypt` trait, which will
store the data in a protobuf `EncryptedMessage` structure.

## Topics

The `define_topics!` macro is a helper to generate a nested tree of topics defined by
path, topic string (with variables), and a protobuf message type to initiate the request.

A `topic!` macro is generated, which allows for retrieving the topic wrapper struct (generated
by the `define_topics!` macro) via its path.

Each topic wrapper struct can generate a topic string, where the number of arguments is
checked at compile time, as well as return the initial request message struct.

## Handlers

## Keys

