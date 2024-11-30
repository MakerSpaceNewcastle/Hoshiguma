//! This build script copies the `memory.x` file from the crate root into
//! a directory where the linker can always find it at build time.
//! For many projects this is optional, as the linker always searches the
//! project root directory -- wherever `Cargo.toml` is. However, if you
//! are using a workspace or have a more complicated build setup, this
//! build script becomes required. Additionally, by requesting that
//! Cargo re-run the build script whenever `memory.x` is changed,
//! updating `memory.x` ensures a rebuild of the application with the
//! new memory settings.

use serde::Deserialize;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    // Put `memory.x` in our output directory and ensure it's
    // on the linker search path.
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(include_bytes!("memory.x"))
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());

    // By default, Cargo will re-run a build script whenever
    // any file in the project changes. By specifying `memory.x`
    // here, we ensure the build script is only re-run when
    // `memory.x` is changed.
    println!("cargo:rerun-if-changed=memory.x");

    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    println!("cargo:rustc-link-arg-bins=-Tlink-rp.x");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");

    // Load the config for the specific controller and set the required environment variables
    Config::load_and_set();
}

#[derive(Deserialize)]
struct Config {
    wifi_ssid: String,
    wifi_password: String,

    mqtt_broker_url: String,
    mqtt_client_id: String,

    online_mqtt_topic: String,
    version_mqtt_topic: String,
    telemetry_mqtt_topic: String,
}

impl Config {
    fn load_and_set() {
        let config = Self::load(env!("CONFIG"));
        config.set_env_vars();
    }

    fn load(filename: &str) -> Self {
        let config = std::fs::read_to_string(filename).unwrap();
        toml::from_str(&config).unwrap()
    }

    fn set_env_vars(&self) {
        println!("cargo::rustc-env=WIFI_SSID={}", self.wifi_ssid);
        println!("cargo::rustc-env=WIFI_PASSWORD={}", self.wifi_password);
        println!("cargo::rustc-env=MQTT_BROKER_URL={}", self.mqtt_broker_url);
        println!("cargo::rustc-env=MQTT_CLIENT_ID={}", self.mqtt_client_id);
        println!(
            "cargo::rustc-env=ONLINE_MQTT_TOPIC={}",
            self.online_mqtt_topic
        );
        println!(
            "cargo::rustc-env=VERSION_MQTT_TOPIC={}",
            self.version_mqtt_topic
        );
        println!(
            "cargo::rustc-env=TELEMETRY_MQTT_TOPIC={}",
            self.telemetry_mqtt_topic
        );
    }
}
