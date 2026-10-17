fn main() {
    println!("cargo:rerun-if-env-changed=LIBXML2_NO_PKG_CONFIG");
    if let Err(error) = pkg_config::Config::new()
        .atleast_version("2.9.0")
        .probe("libxml-2.0")
    {
        eprintln!(
            "libxml2 2.9 or newer is required to build the isolated validation helper: {error}"
        );
        std::process::exit(1);
    }
}
