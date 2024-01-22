/*
 * Up API
 *
 * The Up API gives you programmatic access to your balances and transaction data. You can request past transactions or set up webhooks to receive real-time events when new transactions hit your account. It’s new, it’s exciting and it’s just the beginning. 
 *
 * The version of the OpenAPI document: v1
 * 
 * Generated by: https://openapi-generator.tech
 */


use reqwest;

use crate::apis::ResponseContent;
use super::{Error, configuration};

/// struct for passing parameters to the method [`tags_get`]
#[derive(Clone, Debug)]
pub struct TagsGetParams {
    /// The number of records to return in each page. 
    pub page_left_square_bracket_size_right_square_bracket: Option<i32>
}

/// struct for passing parameters to the method [`transactions_transaction_id_relationships_tags_delete`]
#[derive(Clone, Debug)]
pub struct TransactionsTransactionIdRelationshipsTagsDeleteParams {
    /// The unique identifier for the transaction. 
    pub transaction_id: String,
    pub update_transaction_tags_request: Option<crate::models::UpdateTransactionTagsRequest>
}

/// struct for passing parameters to the method [`transactions_transaction_id_relationships_tags_post`]
#[derive(Clone, Debug)]
pub struct TransactionsTransactionIdRelationshipsTagsPostParams {
    /// The unique identifier for the transaction. 
    pub transaction_id: String,
    pub update_transaction_tags_request: Option<crate::models::UpdateTransactionTagsRequest>
}


/// struct for typed errors of method [`tags_get`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TagsGetError {
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`transactions_transaction_id_relationships_tags_delete`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TransactionsTransactionIdRelationshipsTagsDeleteError {
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`transactions_transaction_id_relationships_tags_post`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TransactionsTransactionIdRelationshipsTagsPostError {
    UnknownValue(serde_json::Value),
}


/// Retrieve a list of all tags currently in use. The returned list is [paginated](#pagination) and can be scrolled by following the `next` and `prev` links where present. Results are ordered lexicographically. The `transactions` relationship for each tag exposes a link to get the transactions with the given tag. 
pub async fn tags_get(configuration: &configuration::Configuration, params: TagsGetParams) -> Result<crate::models::ListTagsResponse, Error<TagsGetError>> {
    let local_var_configuration = configuration;

    // unbox the parameters
    let page_left_square_bracket_size_right_square_bracket = params.page_left_square_bracket_size_right_square_bracket;


    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/tags", local_var_configuration.base_path);
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    if let Some(ref local_var_str) = page_left_square_bracket_size_right_square_bracket {
        local_var_req_builder = local_var_req_builder.query(&[("page[size]", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    if let Some(ref local_var_token) = local_var_configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(local_var_token.to_owned());
    };

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<TagsGetError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Disassociates one or more tags from a specific transaction. Tags that are not associated are silently ignored. An HTTP `204` is returned on success. The associated tags, along with this request URL, are also exposed via the `tags` relationship on the transaction resource returned from `/transactions/{id}`. 
pub async fn transactions_transaction_id_relationships_tags_delete(configuration: &configuration::Configuration, params: TransactionsTransactionIdRelationshipsTagsDeleteParams) -> Result<(), Error<TransactionsTransactionIdRelationshipsTagsDeleteError>> {
    let local_var_configuration = configuration;

    // unbox the parameters
    let transaction_id = params.transaction_id;
    let update_transaction_tags_request = params.update_transaction_tags_request;


    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/transactions/{transactionId}/relationships/tags", local_var_configuration.base_path, transactionId=crate::apis::urlencode(transaction_id));
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::DELETE, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    if let Some(ref local_var_token) = local_var_configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(local_var_token.to_owned());
    };
    local_var_req_builder = local_var_req_builder.json(&update_transaction_tags_request);

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        Ok(())
    } else {
        let local_var_entity: Option<TransactionsTransactionIdRelationshipsTagsDeleteError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Associates one or more tags with a specific transaction. No more than 6 tags may be present on any single transaction. Duplicate tags are silently ignored. An HTTP `204` is returned on success. The associated tags, along with this request URL, are also exposed via the `tags` relationship on the transaction resource returned from `/transactions/{id}`. 
pub async fn transactions_transaction_id_relationships_tags_post(configuration: &configuration::Configuration, params: TransactionsTransactionIdRelationshipsTagsPostParams) -> Result<(), Error<TransactionsTransactionIdRelationshipsTagsPostError>> {
    let local_var_configuration = configuration;

    // unbox the parameters
    let transaction_id = params.transaction_id;
    let update_transaction_tags_request = params.update_transaction_tags_request;


    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/transactions/{transactionId}/relationships/tags", local_var_configuration.base_path, transactionId=crate::apis::urlencode(transaction_id));
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::POST, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    if let Some(ref local_var_token) = local_var_configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(local_var_token.to_owned());
    };
    local_var_req_builder = local_var_req_builder.json(&update_transaction_tags_request);

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        Ok(())
    } else {
        let local_var_entity: Option<TransactionsTransactionIdRelationshipsTagsPostError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}
