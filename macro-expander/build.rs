pub fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-check-cfg=cfg(has_tracked_env_var)");

    #[cfg(feature = "enable")]
    {
        let cfg = autocfg::AutoCfg::new().unwrap();

        if cfg.probe_raw(TRACKED_ENV_VAR_PROBE).is_ok() {
            autocfg::emit("has_tracked_env_var");
        }
    }
    let _ = TRACKED_ENV_VAR_PROBE;
}

const TRACKED_ENV_VAR_PROBE: &str = r##"
#![feature(proc_macro_tracked_env)]
extern crate proc_macro;

fn env_var(name: &str) -> Result<String, std::env::VarError> {
    proc_macro::tracked::env_var(name)
}
"##;
