use git2::Repository;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Metadata {
    name: String,
    symbol: String,
    decimals: u8,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ContractInfo {
    contract_address: String,
    image_url: String,
    metadata: Metadata,
}

fn main() {
    // 1. Define the repository and directory to clone
    let repo_url = "https://github.com/chainapsis/keplr-contract-registry.git";
    let repo_dir = "keplr-contract-registry";
    let secret_dir = "cosmos/secret/tokens";

    // 2. Set the output directory
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let clone_dir = out_dir.join(repo_dir);

    // 3. Clone the repository (shallow clone)
    if !clone_dir.exists() {
        println!("Cloning repository...");
        Repository::clone(repo_url, &clone_dir).expect("Failed to clone repository");
    }

    // 4. Define the directory to work with
    let tokens_dir = clone_dir.join(secret_dir);

    // 5. Iterate over all JSON files in the `tokens` directory
    let mut token_map = std::collections::HashMap::new();

    for entry in fs::read_dir(&tokens_dir).expect("Failed to read tokens directory") {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();

        println!("{:?}", entry);

        if path.extension() == Some(std::ffi::OsStr::new("json")) {
            let file_name = path.file_stem().unwrap().to_str().unwrap().to_string();
            let file_content = fs::read_to_string(&path).expect("Failed to read file");

            // Deserialize the file content into a ContractInfo struct
            let contract_info: ContractInfo = serde_json::from_str(&file_content)
                .expect("Failed to parse JSON into ContractInfo");

            // Insert the ContractInfo struct into the HashMap
            token_map.insert(file_name, contract_info);
        }
    }

    // 6. Serialize the HashMap and write it to the build directory
    let serialized = serde_json::to_string(&token_map).expect("Failed to serialize HashMap");
    let map_file_path = out_dir.join("token_map.json");
    fs::write(&map_file_path, serialized).expect("Failed to write token_map.json");

    // 7. Instruct Cargo to re-run the build script if any files in the tokens directory change
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", tokens_dir.to_str().unwrap());
}
