use std::env;

fn main() {
    for (variable, config) in [
        ("CATLIONS_CONNECT_TEST", "catlions_connect_test"),
        ("CATLIONS_CONNECT_REFUSED", "catlions_connect_refused"),
        ("CATLIONS_PROBE_APP", "catlions_probe_app"),
    ] {
        println!("cargo:rerun-if-env-changed={variable}");
        println!("cargo:rustc-check-cfg=cfg({config})");
        if env::var(variable).as_deref() == Ok("1") {
            println!("cargo:rustc-cfg={config}");
        }
    }
}
