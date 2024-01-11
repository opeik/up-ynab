use chrono::{DateTime, Utc};
use iso_currency::Currency;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionStatus {
    Held,
    Settled,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CardPurchaseKind {
    BarCode,
    Ocr,
    CardPin,
    CardDetails,
    CardOnFile,
    Ecommerce,
    MagneticStripe,
    Contactless,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transactions {
    /// The list of transactions returned in this response.
    data: Vec<TransactionResource>,
    links: PageLinks,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionResource {
    /// The type of this resource: `transactions`.
    #[serde(rename = "type")]
    kind: String,
    /// The unique identifier for this transaction.
    id: String,
    attributes: Attributes,
    relationships: Relationships,
    links: Links,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageLinks {
    /// The link to the previous page in the results. If this value is `null` there is no
    /// previous page.
    prev: Option<Url>,
    /// The link to the next page in the results. If this value is `null` there is no next
    /// page.
    next: Option<Url>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Money {
    /// The ISO 4217 currency code.
    currency_code: Currency,
    /// The amount of money, formatted as a string in the relevant currency. For example, for
    /// an Australian dollar value of $10.56, this field will be "10.56". The currency symbol
    /// is not included in the string.
    value: String,
    /// The amount of money in the smallest denomination for the currency, as a 64-bit integer.
    /// For example, for an Australian dollar value of $10.56, this field will be 1056.
    value_in_base_units: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HoldInfo {
    /// The amount of this transaction while in the HELD status, in Australian dollars.
    amount: Money,
    /// The foreign currency amount of this transaction while in the HELD status. This field
    /// will be null for domestic transactions. The amount was converted to the AUD amount
    /// reflected in the amount field.
    foreign_amount: Option<Money>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoundUp {
    /// The total amount of this Round Up, including any boosts, represented as a negative
    /// value.
    amount: Money,
    /// The portion of the Round Up amount owing to boosted Round Ups, represented as a
    /// negative value. If no boost was added to the Round Up this field will be null.
    boost_portion: Option<Money>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cashback {
    /// A brief description of why this cashback was paid.
    description: String,
    /// The total amount of cashback paid, represented as a positive value.
    amount: Money,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardPurchase {
    /// The type of card purchase.
    method: CardPurchaseKind,
    /// The last four digits of the card used for the purchase, if applicable.
    card_number_suffix: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attributes {
    /// The current processing status of this transaction, according to whether or not this
    /// transaction has settled or is still held.
    status: TransactionStatus,
    /// The original, unprocessed text of the transaction. This is often not a perfect
    /// indicator of the actual merchant, but it is useful for reconciliation purposes in some
    /// cases.
    raw_text: Option<String>,
    /// A short description for this transaction. Usually the merchant name for purchases.
    description: String,
    /// Attached message for this transaction, such as a payment message, or a transfer note.
    message: Option<String>,
    /// Boolean flag set to true on transactions that support the use of categories.
    is_categorizable: bool,
    /// If this transaction is currently in the `HELD` status, or was ever in the `HELD`
    /// status, the amount and foreignAmount of the transaction while `HELD``.
    hold_info: Option<HoldInfo>,
    /// Details of how this transaction was rounded-up. If no Round Up was applied this field
    /// will be `null`.
    round_up: Option<RoundUp>,
    /// If all or part of this transaction was instantly reimbursed in the form of cashback,
    /// details of the reimbursement.
    cashback: Option<Cashback>,
    /// The amount of this transaction in Australian dollars. For transactions that were once
    /// `HELD` but are now `SETTLED`, refer to the `holdInfo` field for the original amount the
    /// transaction was `HELD` at.
    amount: Money,
    /// The foreign currency amount of this transaction. This field will be `null` for domestic
    /// transactions. The amount was converted to the AUD amount reflected in the amount of
    /// this transaction. Refer to the `holdInfo` field for the original `foreignAmount` the
    /// transaction was `HELD` at.
    foreign_amount: Option<Money>,
    /// Information about the card used for this transaction, if applicable.
    card_purchase_method: Option<CardPurchase>,
    /// The date-time at which this transaction settled. This field will be `null` for
    /// transactions that are currently in the `HELD` status.
    settled_at: Option<DateTime<Utc>>,
    /// The date-time at which this transaction was first encountered.
    created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    #[serde(rename = "type")]
    kind: String,
    id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Links {
    /// The canonical link to this resource within the API.
    #[serde(rename = "self")]
    this: Option<Url>,
    /// The link to retrieve the related resource(s) in this relationship.
    related: Option<Url>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reference {
    data: Option<Data>,
    /// The link to retrieve the related resource(s) in this relationship.
    links: Option<Links>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagData {
    /// The type of this resource.
    #[serde(rename = "type")]
    kind: String,
    /// The unique identifier of the resource within its type.
    id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagLinks {
    #[serde(rename = "self")]
    this: Url,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tags {
    data: Vec<TagData>,
    links: Option<TagLinks>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Relationships {
    account: Reference,
    /// If this transaction is a transfer between accounts, this relationship will contain the
    /// account the transaction went to/came from. The amount field can be used to determine
    /// the direction of the transfer.
    transfer_account: Reference,
    category: Reference,
    parent_category: Reference,
    tags: Tags,
}
