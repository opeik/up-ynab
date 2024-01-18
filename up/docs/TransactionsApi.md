# \TransactionsApi

All URIs are relative to *https://api.up.com.au/api/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**accounts_account_id_transactions_get**](TransactionsApi.md#accounts_account_id_transactions_get) | **GET** /accounts/{accountId}/transactions | List transactions by account
[**transactions_get**](TransactionsApi.md#transactions_get) | **GET** /transactions | List transactions
[**transactions_id_get**](TransactionsApi.md#transactions_id_get) | **GET** /transactions/{id} | Retrieve transaction



## accounts_account_id_transactions_get

> crate::models::ListTransactionsResponse accounts_account_id_transactions_get(account_id, page_left_square_bracket_size_right_square_bracket, filter_left_square_bracket_status_right_square_bracket, filter_left_square_bracket_since_right_square_bracket, filter_left_square_bracket_until_right_square_bracket, filter_left_square_bracket_category_right_square_bracket, filter_left_square_bracket_tag_right_square_bracket)
List transactions by account

Retrieve a list of all transactions for a specific account. The returned list is [paginated](#pagination) and can be scrolled by following the `next` and `prev` links where present. To narrow the results to a specific date range pass one or both of `filter[since]` and `filter[until]` in the query string. These filter parameters **should not** be used for pagination. Results are ordered newest first to oldest last. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_id** | **String** | The unique identifier for the account.  | [required] |
**page_left_square_bracket_size_right_square_bracket** | Option<**i32**> | The number of records to return in each page.  |  |
**filter_left_square_bracket_status_right_square_bracket** | Option<[**TransactionStatusEnum**](.md)> | The transaction status for which to return records. This can be used to filter `HELD` transactions from those that are `SETTLED`.  |  |
**filter_left_square_bracket_since_right_square_bracket** | Option<**String**> | The start date-time from which to return records, formatted according to rfc-3339. Not to be used for pagination purposes.  |  |
**filter_left_square_bracket_until_right_square_bracket** | Option<**String**> | The end date-time up to which to return records, formatted according to rfc-3339. Not to be used for pagination purposes.  |  |
**filter_left_square_bracket_category_right_square_bracket** | Option<**String**> | The category identifier for which to filter transactions. Both parent and child categories can be filtered through this parameter. Providing an invalid category identifier results in a `404` response.  |  |
**filter_left_square_bracket_tag_right_square_bracket** | Option<**String**> | A transaction tag to filter for which to return records. If the tag does not exist, zero records are returned and a success response is given.  |  |

### Return type

[**crate::models::ListTransactionsResponse**](ListTransactionsResponse.md)

### Authorization

[bearer_auth](../README.md#bearer_auth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## transactions_get

> crate::models::ListTransactionsResponse transactions_get(page_left_square_bracket_size_right_square_bracket, filter_left_square_bracket_status_right_square_bracket, filter_left_square_bracket_since_right_square_bracket, filter_left_square_bracket_until_right_square_bracket, filter_left_square_bracket_category_right_square_bracket, filter_left_square_bracket_tag_right_square_bracket)
List transactions

Retrieve a list of all transactions across all accounts for the currently authenticated user. The returned list is [paginated](#pagination) and can be scrolled by following the `next` and `prev` links where present. To narrow the results to a specific date range pass one or both of `filter[since]` and `filter[until]` in the query string. These filter parameters **should not** be used for pagination. Results are ordered newest first to oldest last. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**page_left_square_bracket_size_right_square_bracket** | Option<**i32**> | The number of records to return in each page.  |  |
**filter_left_square_bracket_status_right_square_bracket** | Option<[**TransactionStatusEnum**](.md)> | The transaction status for which to return records. This can be used to filter `HELD` transactions from those that are `SETTLED`.  |  |
**filter_left_square_bracket_since_right_square_bracket** | Option<**String**> | The start date-time from which to return records, formatted according to rfc-3339. Not to be used for pagination purposes.  |  |
**filter_left_square_bracket_until_right_square_bracket** | Option<**String**> | The end date-time up to which to return records, formatted according to rfc-3339. Not to be used for pagination purposes.  |  |
**filter_left_square_bracket_category_right_square_bracket** | Option<**String**> | The category identifier for which to filter transactions. Both parent and child categories can be filtered through this parameter. Providing an invalid category identifier results in a `404` response.  |  |
**filter_left_square_bracket_tag_right_square_bracket** | Option<**String**> | A transaction tag to filter for which to return records. If the tag does not exist, zero records are returned and a success response is given.  |  |

### Return type

[**crate::models::ListTransactionsResponse**](ListTransactionsResponse.md)

### Authorization

[bearer_auth](../README.md#bearer_auth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## transactions_id_get

> crate::models::GetTransactionResponse transactions_id_get(id)
Retrieve transaction

Retrieve a specific transaction by providing its unique identifier. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | The unique identifier for the transaction.  | [required] |

### Return type

[**crate::models::GetTransactionResponse**](GetTransactionResponse.md)

### Authorization

[bearer_auth](../README.md#bearer_auth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

