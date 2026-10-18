fn main() {
    println!("cargo:rerun-if-env-changed=LIBXML2_NO_PKG_CONFIG");
    println!("cargo:rerun-if-env-changed=O2XML_STATIC_XML_LIBS");

    #[cfg(all(target_os = "windows", target_env = "msvc"))]
    {
        probe_vcpkg("libxml2");
        probe_vcpkg("libxslt");
        return;
    }

    let static_link = std::env::var_os("O2XML_STATIC_XML_LIBS").is_some();
    probe_pkg_config("libxml-2.0", "2.9.0", static_link);
    probe_pkg_config("libxslt", "1.1.0", static_link);
}

#[cfg(all(target_os = "windows", target_env = "msvc"))]
fn probe_vcpkg(package: &str) {
    if let Err(error) = vcpkg::Config::new().find_package(package) {
        eprintln!("{package} is required to build the isolated helper: {error}");
        std::process::exit(1);
    }
}

#[cfg(not(all(target_os = "windows", target_env = "msvc")))]
fn probe_pkg_config(package: &str, minimum_version: &str, static_link: bool) {
    if let Err(error) = pkg_config::Config::new()
        .atleast_version(minimum_version)
        .statik(static_link)
        .probe(package)
    {
        eprintln!(
            "{package} {minimum_version} or newer is required to build the isolated helper: {error}"
        );
        std::process::exit(1);
    }
}
