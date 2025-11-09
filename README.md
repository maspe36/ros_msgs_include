# `include!()` Generated Sources
This approach works and seems to work well at that. 
Essentially, we have a simple lib.rs which just includes a generated source file.
That generated source file, then `include!()`s all found `.rs` files in whatever directory 
we find via the ament_prefix env var.

1. Determine if the ROS 2 package which holds the `.msg`, `.srv`, and/or `.action` files has been sourced by inspecting the AMENT_PREFIX env var.
2. Confirm that the rust source code has been generated
3. Crawl through the generated source code directory add this line, `include!(<path>);` in a generated source file, for each `.rs` file found.

You should have a generated file, local to this crate, that looks like this
```rust
// env!("OUT_DIR") / shim.rs

pub mod msg_package {
    include!("path/to/msg");
    include!("path/to/action");
}
```

4. `include!()` this local generated source file and re-export any symbols
```rust
include!(concat!(env!("OUT_DIR"), "/shim.rs"));

pub use msg_package::*;
```

5. Pure cargo ROS 2 packages can now use `msg_package` via the shim crate by specifying it in the Cargo.toml
Note, the version number would need to bump anytime there is a major update to the rosidl_runtime_rs crate or 
changes to the deps of generated crates from rosidl_rust.
```toml
[dependencies]
msg_package = "0.0.1"
```

# Pros
- Users have a well-defined package that they can pull from crates.io (or some other registry)
- Doesn't interfere with the message generation pipeline
- No patching required (for messages)

# Cons
- Unusual logic to verify the ROS generated rust source code has all of its dependencies met in the shim message crate
- This is not exactly a well-defined pattern in Rust

# Notes
- Any changes to the referenced environment variables (via `env!()`) automatically trigger a rebuild of the build.rs. 
  - So for example, if the user sources additional workspaces, the shim crates will re-run their build scripts.
  - https://doc.rust-lang.org/cargo/reference/build-scripts.html#rerun-if-env-changed
- This doesn't help the "one source of truth" problem for rust sources.
- Currently, there is no support for a "shim" message crate in a workspace. The best working idea I have is to expect 
the user to wrap their message package (which is normally an `ament_cmake` package), in a `build.rs` that invokes CMake 
and then this macro.