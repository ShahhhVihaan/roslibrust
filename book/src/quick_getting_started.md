# Quick Getting Started

Brief guide for setting up roslibrust, if you get stuck or want a more in depth guide checkout [Extended Getting Started](./extended_getting_started.md)

Add roslibrust as a dependency in `Cargo.toml`:

```toml
{{#include ../../example_package/Cargo.toml}}
```

Setup a `build.rs` file to generate ROS types, and set your `search_paths` to point at your ROS messages:

```rust,ignore
{{#include ../../example_package/build.rs}}
```

Write a basic generic node with tests, and profit:

```rust,ignore
{{#include ../../example_package/src/main.rs}}
```
