# TransactionResourceAttributes

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**status** | [**crate::models::TransactionStatusEnum**](TransactionStatusEnum.md) |  | 
**raw_text** | Option<**String**> | The original, unprocessed text of the transaction. This is often not a perfect indicator of the actual merchant, but it is useful for reconciliation purposes in some cases.  | 
**description** | **String** | A short description for this transaction. Usually the merchant name for purchases.  | 
**message** | Option<**String**> | Attached message for this transaction, such as a payment message, or a transfer note.  | 
**is_categorizable** | **bool** | Boolean flag set to true on transactions that support the use of categories.  | 
**hold_info** | Option<[**crate::models::TransactionResourceAttributesHoldInfo**](TransactionResource_attributes_holdInfo.md)> |  | 
**round_up** | Option<[**crate::models::TransactionResourceAttributesRoundUp**](TransactionResource_attributes_roundUp.md)> |  | 
**cashback** | Option<[**crate::models::TransactionResourceAttributesCashback**](TransactionResource_attributes_cashback.md)> |  | 
**amount** | [**crate::models::MoneyObject**](MoneyObject.md) |  | 
**foreign_amount** | Option<[**crate::models::TransactionResourceAttributesForeignAmount**](TransactionResource_attributes_foreignAmount.md)> |  | 
**settled_at** | Option<**String**> | The date-time at which this transaction settled. This field will be `null` for transactions that are currently in the `HELD` status.  | 
**created_at** | **String** | The date-time at which this transaction was first encountered.  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


