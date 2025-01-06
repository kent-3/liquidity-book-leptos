use base64::prelude::{Engine as _, BASE64_STANDARD};
use cosmwasm_std::{Addr, ContractInfo};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::sync::LazyLock;
use tracing::{debug, info, trace};

pub struct BatchQueryRouter {
    pub localsecret: ContractInfo,
    pub pulsar: ContractInfo,
    pub mainnet: ContractInfo,
}

// pub static BATCH_QUERY_ROUTER: LazyLock<ContractInfo> = LazyLock::new(|| {
//     if CHAIN_ID == "secretdev-1" {
//         ContractInfo {
//             // FIXME: this address needs to be updated manually
//             address: Addr::unchecked("secret1rgqxfst0frq5mgmw3e5pzajpre4qwepc2uh22m"),
//             code_hash: "1c7e86ba4fdb6760e70bf08a7df7f44b53eb0b23290e3e69ca96140810d4f432"
//                 .to_string(),
//         }
//     } else if CHAIN_ID == "pulsar-3" {
//         ContractInfo {
//             address: Addr::unchecked("secret19a9emj5ym504a5824vc7g5awaj2z5nwsl8jpcz"),
//             code_hash: "1c7e86ba4fdb6760e70bf08a7df7f44b53eb0b23290e3e69ca96140810d4f432"
//                 .to_string(),
//         }
//     } else {
//         ContractInfo {
//             address: Addr::unchecked("secret15mkmad8ac036v4nrpcc7nk8wyr578egt077syt"),
//             code_hash: "1c7e86ba4fdb6760e70bf08a7df7f44b53eb0b23290e3e69ca96140810d4f432"
//                 .to_string(),
//         }
//     }
// });

// TODO: figure out how to make this better. it can't rely on a crate-wide static, now that
// it's a separate crate.

pub static BATCH_QUERY_ROUTER: LazyLock<BatchQueryRouter> = LazyLock::new(|| {
    BatchQueryRouter {
        localsecret: ContractInfo {
            // FIXME: this address needs to be updated manually
            address: Addr::unchecked("secret15zvwtzf38yqhdzt2svdk7mnc5ha24493tqydn2"),
            code_hash: "1c7e86ba4fdb6760e70bf08a7df7f44b53eb0b23290e3e69ca96140810d4f432"
                .to_string(),
        },
        pulsar: ContractInfo {
            address: Addr::unchecked("secret19a9emj5ym504a5824vc7g5awaj2z5nwsl8jpcz"),
            code_hash: "1c7e86ba4fdb6760e70bf08a7df7f44b53eb0b23290e3e69ca96140810d4f432"
                .to_string(),
        },
        mainnet: ContractInfo {
            address: Addr::unchecked("secret15mkmad8ac036v4nrpcc7nk8wyr578egt077syt"),
            code_hash: "1c7e86ba4fdb6760e70bf08a7df7f44b53eb0b23290e3e69ca96140810d4f432"
                .to_string(),
        },
    }
});

mod types;
use types::*;

pub use types::{
    BatchItemResponseStatus, BatchQuery, BatchQueryParams, BatchQueryParsedResponse,
    BatchQueryResponse,
};

fn decode_b64_to_json<T: DeserializeOwned>(base64_str: &str) -> T {
    let decoded = BASE64_STANDARD.decode(base64_str).expect("Invalid Base64");
    debug!("{}", String::from_utf8(decoded.clone()).unwrap());
    serde_json::from_slice(&decoded).expect("Invalid JSON")
}

fn decode_b64_to_string(base64_str: &str) -> String {
    let decoded = BASE64_STANDARD.decode(base64_str).expect("Invalid Base64");
    String::from_utf8(decoded).expect("Invalid UTF-8")
}

pub fn msg_batch_query<T: Serialize>(queries: Vec<BatchQueryParams<T>>) -> BatchQuery {
    let batch_queries = queries
        .into_iter()
        .map(|batch_query| BatchQueryItem {
            id: BASE64_STANDARD.encode(&serde_json::to_string(&batch_query.id).unwrap()),
            contract: batch_query.contract,
            query: BASE64_STANDARD.encode(&serde_json::to_string(&batch_query.query_msg).unwrap()),
        })
        .collect();

    BatchQuery {
        batch: Batch {
            queries: batch_queries,
        },
    }
}

pub fn parse_batch_query(response: BatchQueryResponse) -> BatchQueryParsedResponse {
    let responses = response.batch.responses;

    let parsed_items = responses
        .into_iter()
        .map(|item| {
            if let Some(system_err) = item.response.system_err {
                BatchQueryParsedResponseItem {
                    id: decode_b64_to_string(&item.id), // Decode id from Base64
                    response: system_err,               // Directly use the system_err string
                    status: BatchItemResponseStatus::ERROR,
                    block_height: response.batch.block_height,
                }
            } else if let Some(encoded_response) = item.response.response {
                BatchQueryParsedResponseItem {
                    id: decode_b64_to_string(&item.id), // Decode id from Base64
                    response: decode_b64_to_string(&encoded_response), // Decode response from Base64 and JSON
                    status: BatchItemResponseStatus::SUCCESS,
                    block_height: response.batch.block_height,
                }
            } else {
                panic!("Unexpected response format"); // This should not happen
            }
        })
        .collect();

    BatchQueryParsedResponse {
        items: parsed_items,
    }
}
