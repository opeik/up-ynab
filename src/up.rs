mod payload {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    enum TransactionStatusEnum {
        Held,
        Settled,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    enum CardPurchaseMethodEnum {
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
        data: Vec<TransactionResource>,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct TransactionResource {
        #[serde(rename = "type")]
        kind: String,
        id: String,
        attributes: Attributes,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct MoneyObject {
        currency_code: String,
        value: String,
        value_in_base_units: i64,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct HoldInfoObject {
        amount: MoneyObject,
        foreign_amount: Option<MoneyObject>,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RoundUpObject {
        amount: MoneyObject,
        boost_portion: Option<MoneyObject>,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CashbackObject {
        description: String,
        amount: MoneyObject,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CardPurchaseMethodObject {
        method: CardPurchaseMethodEnum,
        card_number_suffix: Option<String>,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Attributes {
        status: TransactionStatusEnum,
        raw_text: Option<String>,
        description: String,
        message: Option<String>,
        is_categorizable: bool,
        hold_info: Option<HoldInfoObject>,
        round_up: Option<RoundUpObject>,
        cashback: Option<CashbackObject>,
        amount: MoneyObject,
        foreign_amount: Option<MoneyObject>,
        card_purchase_method: CardPurchaseMethodObject,
        settled_at: Option<String>,
        created_at: String,
    }
}
