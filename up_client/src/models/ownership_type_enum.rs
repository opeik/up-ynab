/*
 * Up API
 *
 * The Up API gives you programmatic access to your balances and transaction data. You can request past transactions or set up webhooks to receive real-time events when new transactions hit your account. It’s new, it’s exciting and it’s just the beginning. 
 *
 * The version of the OpenAPI document: v1
 * 
 * Generated by: https://openapi-generator.tech
 */

/// OwnershipTypeEnum : Specifies the structure under which a bank account is owned. Currently returned values are `INDIVIDUAL` and `JOINT`. 

/// Specifies the structure under which a bank account is owned. Currently returned values are `INDIVIDUAL` and `JOINT`. 
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum OwnershipTypeEnum {
    #[serde(rename = "INDIVIDUAL")]
    Individual,
    #[serde(rename = "JOINT")]
    Joint,

}

impl ToString for OwnershipTypeEnum {
    fn to_string(&self) -> String {
        match self {
            Self::Individual => String::from("INDIVIDUAL"),
            Self::Joint => String::from("JOINT"),
        }
    }
}

impl Default for OwnershipTypeEnum {
    fn default() -> OwnershipTypeEnum {
        Self::Individual
    }
}



