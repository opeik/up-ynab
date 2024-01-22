/*
 * Up API
 *
 * The Up API gives you programmatic access to your balances and transaction data. You can request past transactions or set up webhooks to receive real-time events when new transactions hit your account. It’s new, it’s exciting and it’s just the beginning. 
 *
 * The version of the OpenAPI document: v1
 * 
 * Generated by: https://openapi-generator.tech
 */

/// WebhookDeliveryLogResource : Provides historical webhook event delivery information for analysis and debugging purposes. 



#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WebhookDeliveryLogResource {
    /// The type of this resource: `webhook-delivery-logs`
    #[serde(rename = "type")]
    pub r#type: String,
    /// The unique identifier for this log entry. 
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "attributes")]
    pub attributes: Box<crate::models::WebhookDeliveryLogResourceAttributes>,
    #[serde(rename = "relationships")]
    pub relationships: Box<crate::models::WebhookDeliveryLogResourceRelationships>,
}

impl WebhookDeliveryLogResource {
    /// Provides historical webhook event delivery information for analysis and debugging purposes. 
    pub fn new(r#type: String, id: String, attributes: crate::models::WebhookDeliveryLogResourceAttributes, relationships: crate::models::WebhookDeliveryLogResourceRelationships) -> WebhookDeliveryLogResource {
        WebhookDeliveryLogResource {
            r#type,
            id,
            attributes: Box::new(attributes),
            relationships: Box::new(relationships),
        }
    }
}

