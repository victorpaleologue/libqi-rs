# libqi-rs

A Rust implementation of the `qi` framework, the middleware of Aldebaran's NAO and
Pepper robots (`libqi`, qimessaging), byte-compatible with the C++ implementation.

With it, Rust programs talk to NAOqi robots and to any `libqi` process: they call
services, subscribe to signals, read and write properties, expose their own services and
objects, or even host a service directory. The wire format is verified byte for byte
against `libqi` 4.0.5, and the implementation is exercised against C++ `libqi`
processes as client, service and service directory.

## Crates

| Crate | Description |
|---|---|
| [`qi`](qi/) | The framework: nodes, sessions, objects, signals, properties, services, the service directory, and the `#[qi::object]` macro. Start here. |
| [`qi-value`](qi-value/) | The type system: types, signatures, dynamic values and conversions. |
| [`qi-format`](qi-format/) | The binary serialization format, as a `serde` data format. |
| [`qi-messaging`](qi-messaging/) | The messaging protocol: messages, channels, client and server loops. |
| [`qi-macros`](qi-macros/) | Procedural macros (`Valuable` derives, `#[qi::object]`). |
| [`qi-tools`](qi-tools/) | `qi-cli`, a command-line tool to inspect and drive services. |
| [`naoqi-sim`](naoqi-sim/) | A simulated NAOqi robot: the services used by `naoqi_driver2` and robot HALs, without a robot. |

The design is described in [`docs/architecture.md`](docs/architecture.md).

## Quick start

```rust
use qi::ObjectExt;

#[tokio::main]
async fn main() -> qi::Result<()> {
    let node = qi::node::init()
        .connect_to_space("tcp://nao.local:9559".parse().expect("valid address"), None)
        .start()
        .await?;
    let tts = node.service("ALTextToSpeech").await?;
    let () = tts.call("say", "Hello from Rust".to_owned()).await?;
    Ok(())
}
```

Typed interfaces, services and objects are shown in the documentation of the `qi` crate
and in [`examples/`](examples/).

## Status

Implemented and tested against `libqi` 4.0.5:

- binary format and messaging protocol, byte-identical (66 reference fixtures);
- sessions with capability negotiation and user/token authentication;
- calls with cancellation, posts, signals (`registerEvent`/`unregisterEvent`), properties;
- objects passed in both directions, with the special bound-object actions;
- services and a standalone service directory with its signals and relative endpoints;
- callbacks in both NAOqi styles: object passing, and services calling back services
  registered by their clients.

- TLS transports (`tcps://`, as used by NAOqi 2.9 robots) with `libqi`'s semantics.

Not implemented yet: mutual TLS authentication (`tcpsm://`), the `Manageable` statistics
and tracing members, and gateways.

## Building and testing

```sh
cargo build --workspace
cargo test --workspace
```

The interoperability tests against C++ `libqi` (`qi/tests/interop_cpp.rs`) run when the
harness in [`interop/cpp/`](interop/cpp/) is built (see its README); they are skipped
otherwise, and `QI_INTEROP_REQUIRE=1` makes skipping an error.

## License

See [`LICENSE.txt`](LICENSE.txt).
