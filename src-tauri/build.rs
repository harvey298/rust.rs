use std::{process::{Command}, fs};
use anyhow::Result;
use toml::Value;

fn _get_build_profile_name() -> String {
  // The profile name is always the 3rd last part of the path (with 1 based indexing).
  // e.g. /code/core/target/cli/build/my-build-info-9f91ba6f99d7a061/out
  std::env::var("OUT_DIR").unwrap()
      .split(std::path::MAIN_SEPARATOR)
      .nth_back(3)
      .unwrap_or_else(|| "unknown")
      .to_string()
}

fn get_target() -> Result<String> {
  let output = Command::new("rustc")
      .arg("-vV")
      .output().unwrap();
  let output: String = String::from_utf8_lossy(&output.stdout).parse().unwrap();

  let output = output.split("\n").into_iter().collect::<Vec<&str>>();

  let output = output[4];
  let output = output.replace("host: ", "");

  Ok(output)
}

fn main() {
  protobuf_codegen::Codegen::new()
  // Use `protoc` parser, optional.
  .protoc()
  // Use `protoc-bin-vendored` bundled protoc command, optional.
  .protoc_path(&protoc_bin_vendored::protoc_bin_path().unwrap())
  
  // All inputs and imports from the inputs must reside in `includes` directories.
  .includes(&["src/rust_plus/protos"])

  // Inputs must reside in some of include paths.
  .input("src/rust_plus/protos/rustplus.proto")
  // Specify output directory relative to Cargo output directory.
  .cargo_out_dir("protos")
  .run_from_script();

  let config = fs::read_to_string("./pyinstaller-sidecar.toml").unwrap();
  let config: Value = toml::from_str(&config).unwrap();

  let config_build = config["build"].as_table().unwrap();
  
  let binding = config_build["dist"].clone();
  let dist = binding.as_str().unwrap();

  let binding = config_build["source_dir"].clone();
  let source_dir = binding.as_str().unwrap();

  let files_to_build = config_build["source_files"].clone();

  let work_path = format!("{}/python-bins/build",std::env::var("OUT_DIR").unwrap());
  let spec_path = format!("{}/python-bins/spec",std::env::var("OUT_DIR").unwrap());

  for file in files_to_build.as_array().unwrap() {

    let filename = file.as_str().unwrap();
    let file = format!("{source_dir}{filename}");

    println!("cargo:rerun-if-changed={file}");

    let output_file = format!("{}-{}.exe",filename.replace(".py", ""),get_target().unwrap());

    let args = vec!["-y", "-F", "--workpath",work_path.as_str(), "--specpath", &spec_path, "--noconsole" , "--name", &output_file , "--distpath", dist, &file];
    
    // println!("{:#?}", args);

    let mut cmd = Command::new("pyinstaller");
    cmd.args(&args);

    let output = cmd.output().unwrap();

    if !output.status.success() {

      let s = String::from_utf8_lossy(&output.stderr);

      println!("result: {}", s);

      panic!("Failed to compile Python!")
    }

  }

  tauri_build::build()
}
