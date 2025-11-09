use std::path::PathBuf;

/// Given a ROS 2 interface package name (i.e. a package with a `.msg`, `.srv`, `.action`, or `.idl` file),
/// search the `AMENT_PREFIX_PATH` environment variable entries for the path to where the generated rust message crate can be found.
#[doc(hidden)]
pub fn find_generated_rust_crate(ament_prefix_path: &str, pkg_name: &str) -> Option<PathBuf> {
    ament_prefix_path
        .split(':')
        .filter(|x| x.ends_with(pkg_name))
        .map(|x| PathBuf::from(x).join("share").join(pkg_name).join("rust"))
        .find(|x| x.exists())
}

#[macro_export]
macro_rules! ros_msgs_include {
    () => {
        $crate::ros_msgs_include!(env!("CARGO_PKG_NAME"))
    };
    ($pkg_name:expr) => {
        use ::ros_msgs_include::cargo_toml::Manifest;
        use std::path::{Path, PathBuf};
        use std::{fs, io};

        // Collect all `real_create` sources and create a file that `include!`s them for re-export.
        let dest_path =
            Path::new(&std::env::var("OUT_DIR").expect("OUT_DIR not set")).join("shim.rs");

        // The crate this macro is invoked from, is intended to be a shim crate that finds
        // another generated crate with the same name in an upstream workspace.
        let pkg_name = env!("CARGO_PKG_NAME");

        let real_crate_dir = ::ros_msgs_include::find_generated_rust_crate(
            env!("AMENT_PREFIX_PATH"),
            pkg_name,
        )
        .unwrap_or_else(|| panic!("Could not find generated rust crate for \"{pkg_name}\""));

        // TODO Verify dependencies

        // Crawl through the real_crate and collect all (non-lib.rs) sources
        let entries: Vec<_> = fs::read_dir(real_crate_dir.join("src"))
            .expect("Failed to read source directory")
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|e| !e.ends_with("lib.rs"))
            .collect();

        // Generate a module and include! the associated .rs file
        // NOTE: We skip lib.rs as there are additional clippy lints. These need to be defined at
        // the root of the library, and since we are aiming to `include!` these modules,
        // we by definition cannot add additional clippy lints to our shim's lib.rs.
        let lines: String = entries
            .iter()
            .filter_map(|e| {
                e.file_stem()
                    .and_then(|stem| stem.to_str())
                    .map(|stem| format!("pub mod {stem} {{ include!(\"{}\"); }}", e.display()))
            })
            .collect::<Vec<_>>()
            .join("\n");

        fs::write(&dest_path, format!("pub mod {pkg_name} {{ {lines} }}"))
            .unwrap_or_else(|_| panic!("Failed to write to {}", dest_path.display()));
    };
}

#[cfg(test)]
mod tests {
    use std::fs;
    use super::*;

    #[test]
    fn test_find_generated_rust_crate() {
        let temp_dir = tempfile::tempdir().unwrap();
        let pkg_name = "test_msgs";

        // Create the expected directory structure
        let rust_dir = temp_dir
            .path()
            .join("install")
            .join(pkg_name)
            .join("share")
            .join(pkg_name)
            .join("rust");
        fs::create_dir_all(&rust_dir).unwrap();

        let ament_path = format!(
            "{}:{}:/opt/ros/jazzy",
            temp_dir.path().join("install").join("not_test_msgs").display(),
            temp_dir.path().join("install").join(pkg_name).display()
        );
        let path = find_generated_rust_crate(
            &ament_path,
            pkg_name,
        );

        assert_eq!(path, Some(rust_dir));
    }
}
