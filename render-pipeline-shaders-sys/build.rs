use std::{env, fs, path::Path};

use bindgen::callbacks::ParseCallbacks;
use bindgen::Formatter;

#[derive(Debug)]
struct BindgenCallbacks;

impl ParseCallbacks for BindgenCallbacks {
    fn item_name(&self, original_item_name: &str) -> Option<String> {
        if original_item_name.starts_with("Vk") {
            Some(original_item_name.trim_start_matches("Vk").to_string())
        } else if original_item_name.starts_with("PFN_vk") && original_item_name.ends_with("KHR") {
            Some(original_item_name.trim_end_matches("KHR").to_string())
        } else {
            None
        }
    }
}

fn generate_bindings() {
    let mut builder = bindgen::Builder::default();

    #[cfg(feature = "vulkan")]
    {
        builder = builder.raw_line("use ash::vk::*;");
        builder = builder.clang_arg("-DRPS_VK_RUNTIME");
    }

    //HACK: replace file and restore it, because it does not work otherwise
    let rps_vk_runtime_str = fs::read_to_string("vendor/RenderPipelineShaders/include/rps/runtime/vk/rps_vk_runtime.h").unwrap();
    let rps_vk_runtime_bindgen = rps_vk_runtime_str.replace("RPS_IMPL_OPAQUE_HANDLE", "//");
    fs::write("vendor/RenderPipelineShaders/include/rps/runtime/vk/rps_vk_runtime.h", rps_vk_runtime_bindgen).unwrap();

    builder = builder
        .clang_arg("-I./vendor/Vulkan-Headers/include")
        .clang_arg("-I./vendor/RenderPipelineShaders/include")
        .header("vendor/RenderPipelineShaders/include/rps/rps.h")
        .formatter(Formatter::Rustfmt)
        .size_t_is_usize(true)
        .allowlist_function("rps.*")
        .allowlist_function("PFN_rps.*")
        .allowlist_type("Rps.*")
        .parse_callbacks(Box::new(BindgenCallbacks))
        .blocklist_type("Vk.*")
        .blocklist_type("PFN_vk.*")
        .blocklist_type("va_list")
        .raw_line("pub type va_list = *mut u8;")
        .layout_tests(false);

    let bindings_result = builder.generate();

    //HACK: restore original file
    fs::write("vendor/RenderPipelineShaders/include/rps/runtime/vk/rps_vk_runtime.h", &rps_vk_runtime_str).unwrap();

    let bindings = bindings_result.expect("Failed to generate bindings");

    let bindings_str = bindings.to_string();

    fs::create_dir_all("gen").unwrap();
    fs::write(Path::new("gen/bindings.rs"), bindings_str).expect("Failed to write bindings to file");
}

fn main() {
    #[cfg(feature = "d3d11")]
    panic!("Feature D3D11 is not supported at the moment");

    #[cfg(feature = "d3d12")]
    panic!("Feature D3D12 is not supported at the moment");

    let out_dir = cmake::Config::new("vendor/RenderPipelineShaders")
        .build_target("all")
        .define("RpsEnableVulkan", "ON")
        .build()
        .join("build")
        .join("src");

    println!("cargo:rustc-link-search={}", out_dir.display());
    println!("cargo:rustc-link-lib=rps_runtime");
    println!("cargo:rustc-link-lib=rps_core");
    println!("cargo:rustc-link-lib=stdc++");

    generate_bindings();
}
