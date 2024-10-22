use git2::Repository;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env, fs, path::PathBuf};

mod keplr {
    use super::*;

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

    pub fn process_tokens() {
        // 1. Define the repository and directory to clone
        let repo_url = "https://github.com/chainapsis/keplr-contract-registry.git";
        let repo_dir = "keplr-contract-registry";
        let secret_dir = "cosmos/secret/tokens";

        // 2. Set the output directory
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        let clone_dir = out_dir.join(repo_dir);

        // Remove the clone directory if it exists to ensure fresh cloning
        if clone_dir.exists() {
            println!("Removing old repository...");
            fs::remove_dir_all(&clone_dir).expect("Failed to remove old repository");
        }

        // 3. Clone the repository
        println!("Cloning repository...");
        Repository::clone(repo_url, &clone_dir).expect("Failed to clone repository");

        // 4. Define the directory to work with
        let tokens_dir = clone_dir.join(secret_dir);

        // 5. Iterate over all JSON files in the `tokens` directory
        let mut token_map = HashMap::new();

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
        let map_file_path = out_dir.join("keplr_token_map.json");
        fs::write(&map_file_path, serialized).expect("Failed to write keplr_token_map.json");
    }
}
mod secret_foundation {
    use super::*;

    #[derive(Serialize, Deserialize, Debug)]
    struct Asset {
        pub contract_name: String,
        pub visible_asset_name: String,
        pub symbol: String,
        pub decimals: u8,
        pub denom: String,
        pub contract_address: String,
        pub contract_hash: String,
        pub version: String,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct Assets {
        #[serde(rename = "Native")]
        pub native: Vec<Asset>,
        #[serde(rename = "Axelar Bridged Assets")]
        pub axelar_bridged_assets: Vec<Asset>,
        #[serde(rename = "IBC Bridged Assets")]
        pub ibc_bridged_assets: Vec<Asset>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct Token {
        pub contract_address: String,
        pub code_hash: String,
        pub decimals: u8,
        pub name: String,
        pub symbol: String,
        pub display_name: Option<String>,
        pub denom: Option<String>,
        pub version: Option<String>,
    }

    impl Into<Token> for Asset {
        fn into(self) -> Token {
            Token {
                contract_address: self.contract_address,
                code_hash: self.contract_hash,
                decimals: self.decimals,
                name: self.contract_name,
                symbol: self.symbol,
                display_name: Some(self.visible_asset_name),
                denom: Some(self.denom),
                version: Some(self.version),
            }
        }
    }

    pub fn process_tokens() {
        let repo_url = "https://github.com/SecretFoundation/AssetRegistry.git";
        let repo_dir = "SecretFoundation";
        let file = "assets.json";

        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        let clone_dir = out_dir.join(repo_dir);

        // Remove the clone directory if it exists to ensure fresh cloning
        if clone_dir.exists() {
            println!("Removing old repository...");
            fs::remove_dir_all(&clone_dir).expect("Failed to remove old repository");
        }

        println!("Cloning repository...");
        Repository::clone(repo_url, &clone_dir).expect("Failed to clone repository");

        let tokens_dir = clone_dir;
        let file_path = tokens_dir.join(file);
        let file_content = fs::read_to_string(&file_path).expect("Failed to read file");

        let assets: Assets =
            serde_json::from_str(&file_content).expect("Failed to parse JSON into Assets");

        let mut flattened_assets = Vec::new();
        flattened_assets.extend(assets.native);
        flattened_assets.extend(assets.axelar_bridged_assets);
        flattened_assets.extend(assets.ibc_bridged_assets);

        let mut token_map: HashMap<String, Token> = HashMap::new();

        for asset in flattened_assets {
            token_map.insert(asset.contract_address.clone(), asset.into());
        }

        let serialized = serde_json::to_string(&token_map).expect("Failed to serialize HashMap");
        let map_file_path = out_dir.join("sf_token_map.json");
        fs::write(&map_file_path, serialized).expect("Failed to write sf_token_map.json");
    }
}

fn main() {
    keplr::process_tokens();
    secret_foundation::process_tokens();

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=PROFILE");
}
