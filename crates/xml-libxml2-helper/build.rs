fn main() {
    println!("cargo:rerun-if-env-changed=LIBXML2_NO_PKG_CONFIG");
    println!("cargo:rerun-if-env-changed=O2XML_STATIC_XML_LIBS");
    println!("cargo:rerun-if-changed=src/validity_callbacks.c");

    let include_paths = configure_native_libraries();
    cc::Build::new()
        .file("src/validity_callbacks.c")
        .includes(include_paths)
        .warnings(true)
        .compile("o2xml_validity_callbacks");
}

#[cfg(all(target_os = "windows", target_env = "msvc"))]
fn configure_native_libraries() -> Vec<std::path::PathBuf> {
    let mut include_paths = probe_vcpkg("libxml2");
    include_paths.extend(probe_vcpkg("libxslt"));
    include_paths
}

#[cfg(not(all(target_os = "windows", target_env = "msvc")))]
fn configure_native_libraries() -> Vec<std::path::PathBuf> {
    let static_link = std::env::var_os("O2XML_STATIC_XML_LIBS").is_some();
    let mut include_paths = probe_pkg_config("libxml-2.0", "2.9.0", static_link);
    include_paths.extend(probe_pkg_config("libxslt", "1.1.0", static_link));
    include_paths
}

#[cfg(all(target_os = "windows", target_env = "msvc"))]
fn probe_vcpkg(package: &str) -> Vec<std::path::PathBuf> {
    match vcpkg::Config::new().find_package(package) {
        Ok(library) => library.include_paths,
        Err(error) => {
            eprintln!("{package} is required to build the isolated helper: {error}");
            std::process::exit(1);
        }
    }
}

#[cfg(not(all(target_os = "windows", target_env = "msvc")))]
fn probe_pkg_config(
    package: &str,
    minimum_version: &str,
    static_link: bool,
) -> Vec<std::path::PathBuf> {
    match pkg_config::Config::new()
        .atleast_version(minimum_version)
        .statik(static_link)
        .probe(package)
    {
        Ok(library) => library.include_paths,
        Err(error) => {
            eprintln!(
                "{package} {minimum_version} or newer is required to build the isolated helper: {error}"
            );
            std::process::exit(1);
        }
    }
}
