# \AccountsApi

All URIs are relative to *https://api.up.com.au/api/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**accounts_get**](AccountsApi.md#accounts_get) | **GET** /accounts | List accounts
[**accounts_id_get**](AccountsApi.md#accounts_id_get) | **GET** /accounts/{id} | Retrieve account



## accounts_get

> crate::models::ListAccountsResponse accounts_get(page_left_square_bracket_size_right_square_bracket, filter_left_square_bracket_account_type_right_square_bracket, filter_left_square_bracket_ownership_type_right_square_bracket)
List accounts

Retrieve a paginated list of all accounts for the currently authenticated user. The returned list is paginated and can be scrolled by following the `prev` and `next` links where present. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**page_left_square_bracket_size_right_square_bracket** | Option<**i32**> | The number of records to return in each page.  |  |
**filter_left_square_bracket_account_type_right_square_bracket** | Option<[**AccountTypeEnum**](.md)> | The type of account for which to return records. This can be used to filter Savers from spending accounts.  |  |
**filter_left_square_bracket_ownership_type_right_square_bracket** | Option<[**OwnershipTypeEnum**](.md)> | The account ownership structure for which to return records. This can be used to filter 2Up accounts from Up accounts.  |  |

### Return type

[**crate::models::ListAccountsResponse**](ListAccountsResponse.md)

### Authorization

[bearer_auth](../README.md#bearer_auth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## accounts_id_get

> crate::models::GetAccountResponse accounts_id_get(id)
Retrieve account

Retrieve a specific account by providing its unique identifier. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | The unique identifier for the account.  | [required] |

### Return type

[**crate::models::GetAccountResponse**](GetAccountResponse.md)

### Authorization

[bearer_auth](../README.md#bearer_auth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

