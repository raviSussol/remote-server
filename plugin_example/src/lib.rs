use std::ffi::CStr;

use plugin_api::{export_plugin, MSupplyPlugin};

#[repr(C)]
pub struct ExamplePlugin {}

impl ExamplePlugin {
    pub fn new() -> Box<Self> {
        println!("ExamplePlugin created");
        Box::new(ExamplePlugin {})
    }
}

impl MSupplyPlugin for ExamplePlugin {
    fn on_load(&self) -> () {
        println!("ExamplePlugin loaded");
    }

    fn on_unload(&self) -> () {
        println!("ExamplePlugin unloaded");
    }

    fn test(
        &self,
        value: i32,
        string: String,
        complex_params: &plugin_api::TestParameter,
    ) -> plugin_api::TestResponse {
        println!("ExamplePlugin test, message from host: {}", string);
        return plugin_api::TestResponse {
            result: value + 5,
            result_struct: plugin_api::TestParameter {
                value: complex_params.value + 5,
            },
        };
    }
}

export_plugin!(ExamplePlugin);
