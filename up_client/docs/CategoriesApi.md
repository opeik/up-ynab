# \CategoriesApi

All URIs are relative to *https://api.up.com.au/api/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**categories_get**](CategoriesApi.md#categories_get) | **GET** /categories | List categories
[**categories_id_get**](CategoriesApi.md#categories_id_get) | **GET** /categories/{id} | Retrieve category
[**transactions_transaction_id_relationships_category_patch**](CategoriesApi.md#transactions_transaction_id_relationships_category_patch) | **PATCH** /transactions/{transactionId}/relationships/category | Categorize transaction



## categories_get

> crate::models::ListCategoriesResponse categories_get(filter_left_square_bracket_parent_right_square_bracket)
List categories

Retrieve a list of all categories and their ancestry. The returned list is not paginated. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**filter_left_square_bracket_parent_right_square_bracket** | Option<**String**> | The unique identifier of a parent category for which to return only its children. Providing an invalid category identifier results in a `404` response.  |  |

### Return type

[**crate::models::ListCategoriesResponse**](ListCategoriesResponse.md)

### Authorization

[bearer_auth](../README.md#bearer_auth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## categories_id_get

> crate::models::GetCategoryResponse categories_id_get(id)
Retrieve category

Retrieve a specific category by providing its unique identifier. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | The unique identifier for the category.  | [required] |

### Return type

[**crate::models::GetCategoryResponse**](GetCategoryResponse.md)

### Authorization

[bearer_auth](../README.md#bearer_auth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## transactions_transaction_id_relationships_category_patch

> transactions_transaction_id_relationships_category_patch(transaction_id, update_transaction_category_request)
Categorize transaction

Updates the category associated with a transaction. Only transactions for which `isCategorizable` is set to true support this operation. The `id` is taken from the list exposed on `/categories` and cannot be one of the top-level (parent) categories. To de-categorize a transaction, set the entire `data` key to `null`. An HTTP `204` is returned on success. The associated category, along with its request URL is also exposed via the `category` relationship on the transaction resource returned from `/transactions/{id}`. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**transaction_id** | **String** | The unique identifier for the transaction.  | [required] |
**update_transaction_category_request** | Option<[**UpdateTransactionCategoryRequest**](UpdateTransactionCategoryRequest.md)> |  |  |

### Return type

 (empty response body)

### Authorization

[bearer_auth](../README.md#bearer_auth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

