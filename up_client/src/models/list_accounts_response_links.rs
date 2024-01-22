/*
 * Up API
 *
 * The Up API gives you programmatic access to your balances and transaction data. You can request past transactions or set up webhooks to receive real-time events when new transactions hit your account. It’s new, it’s exciting and it’s just the beginning. 
 *
 * The version of the OpenAPI document: v1
 * 
 * Generated by: https://openapi-generator.tech
 */




#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ListAccountsResponseLinks {
    /// The link to the previous page in the results. If this value is `null` there is no previous page. 
    #[serde(rename = "prev", deserialize_with = "Option::deserialize")]
    pub prev: Option<String>,
    /// The link to the next page in the results. If this value is `null` there is no next page. 
    #[serde(rename = "next", deserialize_with = "Option::deserialize")]
    pub next: Option<String>,
}

impl ListAccountsResponseLinks {
    pub fn new(prev: Option<String>, next: Option<String>) -> ListAccountsResponseLinks {
        ListAccountsResponseLinks {
            prev,
            next,
        }
    }
}

