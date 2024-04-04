
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use serde_json::json;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsonRpcRequestStandard {
    pub path : String,
    pub http_headers : HashMap<String, String>,
    pub path_params : HashMap<String, String>,
    pub body : serde_json::Value,
    pub query_params : HashMap<String, String>
}

impl JsonRpcRequestStandard {
    pub fn new() -> Self {
        JsonRpcRequestStandard {
            path: "".to_string(),
            http_headers: HashMap::new(),
            path_params: HashMap::new(),
            body: serde_json::Value::Null,
            query_params: serde_json::Value::Null,
        }
    }

    pub fn set_path_param(&mut self, key : String, value : String) {
        self.path_params.insert(key, value);
    }

    pub fn set_http_header(&mut self, key : String, value : String) {
        self.http_headers.insert(key, value);
    }

}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
    pub id: serde_json::Value,
}

impl From<JsonRpcRequestStandard> for JsonRpcRequest {
    fn from(standard : JsonRpcRequestStandard) -> Self {
        JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: standard.path.replace("/", "."), // ? This is a naive way to convert a path to a method name
            params: json!({
                "http_headers": standard.http_headers,
                "path_params": standard.path_params,
                "body": standard.body,
                "query": standard.query_params,
            }),
            id: json!(1), // You can customize this as needed
        }
    }
}

impl From<JsonRpcRequest> for JsonRpcRequestStandard {
    fn from(request : JsonRpcRequest) -> Self {
        let params = request.params.as_object().unwrap();
        JsonRpcRequestStandard {
            http_headers: params.get("http_headers").unwrap().as_object().unwrap().clone(),
            path_params: params.get("path_params").unwrap().as_object().unwrap().clone(),
            body: params.get("body").unwrap().clone(),
            query_params: params.get("query").unwrap().clone(),
        }
    }
}

pub trait ToJsonRpc<T> {

    // ? This currently does not need to be part of the trait. It's just this way for organization.
    /// Converts a request to a method name
    async fn request_to_method(&self, request : &T) -> Result<String, anyhow::Error>;

    async fn to_json_rpc_standard(&self, request : T) -> Result<JsonRpcRequestStandard, anyhow::Error>;

    /// Converts a request to a JsonRpcRequest
    async fn to_json_rpc(&self, request : T) -> Result<JsonRpcRequest, anyhow::Error> {
        let standard = self.to_json_rpc_standard(request).await?;
        Ok(JsonRpcRequest::from(standard))
    }

}

#[async_trait::async_trait] // if we don't have this we can't use Box<dyn Forwarder>
pub trait Forwarder<T> {

    async fn forward(&self, json_rpc_request : JsonRpcRequest) -> Result<T, anyhow::Error>;

}

#[async_trait::async_trait] // if we don't have this we can't use Box<dyn Forwarder>
pub trait Middleware<T> {
    async fn apply(&self, request : T) -> Result<T, anyhow::Error>;
}

#[async_trait::async_trait] // if we don't have this we can't use Box<dyn Forwarder>
pub trait Proxy<T> {

    // ? This is async in case we want to pick up a forwarding destination from a database or something
    async fn set_forwarder(&mut self, forwarder : Box<dyn Forwarder<T> + Send + Sync>) -> Result<(), anyhow::Error>;

    async fn serve(&self) -> Result<(), anyhow::Error>;

}