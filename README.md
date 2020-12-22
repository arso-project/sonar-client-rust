# sonar-client-rust

A client for [Sonar](https://github.com/arso-project/sonar) written in Rust.

*WIP*

Notes:
- Currently needs branch *core-refactor* of Sonar
- Uses the HTTP API to communicate with Sonar
- *hrpc* is not used at the moment, but the serde-json messages for the HTTP API are derived from the Protobuf definitions in *sonar-hrpc* (we intend to add a HRPC API to Sonar, then the same message definitions can be used for both HRPC and HTTP).

