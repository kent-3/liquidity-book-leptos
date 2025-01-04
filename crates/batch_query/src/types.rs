use cosmwasm_std::ContractInfo;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct BatchQueryParams<T: Serialize> {
    pub id: String,
    pub contract: ContractInfo,
    pub query_msg: T,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BatchQuery {
    pub batch: Batch,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Batch {
    pub queries: Vec<BatchQueryItem>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BatchQueryItem {
    pub id: String, // base64-encoded
    pub contract: ContractInfo,
    pub query: String, // base64-encoded
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BatchQueryResponse {
    pub batch: BatchResponse,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BatchResponse {
    pub block_height: u64,
    pub responses: Vec<BatchQueryResponseItem>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BatchQueryResponseItem {
    pub id: String,
    pub contract: ContractInfo,
    pub response: Response,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_err: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchQueryParsedResponse {
    pub items: Vec<BatchQueryParsedResponseItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchQueryParsedResponseItem {
    pub id: String,
    pub response: String,
    pub status: BatchItemResponseStatus,
    pub block_height: u64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum BatchItemResponseStatus {
    SUCCESS,
    ERROR,
}
