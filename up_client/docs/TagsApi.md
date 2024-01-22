# \TagsApi

All URIs are relative to *https://api.up.com.au/api/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**tags_get**](TagsApi.md#tags_get) | **GET** /tags | List tags
[**transactions_transaction_id_relationships_tags_delete**](TagsApi.md#transactions_transaction_id_relationships_tags_delete) | **DELETE** /transactions/{transactionId}/relationships/tags | Remove tags from transaction
[**transactions_transaction_id_relationships_tags_post**](TagsApi.md#transactions_transaction_id_relationships_tags_post) | **POST** /transactions/{transactionId}/relationships/tags | Add tags to transaction



## tags_get

> crate::models::ListTagsResponse tags_get(page_left_square_bracket_size_right_square_bracket)
List tags

Retrieve a list of all tags currently in use. The returned list is [paginated](#pagination) and can be scrolled by following the `next` and `prev` links where present. Results are ordered lexicographically. The `transactions` relationship for each tag exposes a link to get the transactions with the given tag. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**page_left_square_bracket_size_right_square_bracket** | Option<**i32**> | The number of records to return in each page.  |  |

### Return type

[**crate::models::ListTagsResponse**](ListTagsResponse.md)

### Authorization

[bearer_auth](../README.md#bearer_auth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## transactions_transaction_id_relationships_tags_delete

> transactions_transaction_id_relationships_tags_delete(transaction_id, update_transaction_tags_request)
Remove tags from transaction

Disassociates one or more tags from a specific transaction. Tags that are not associated are silently ignored. An HTTP `204` is returned on success. The associated tags, along with this request URL, are also exposed via the `tags` relationship on the transaction resource returned from `/transactions/{id}`. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**transaction_id** | **String** | The unique identifier for the transaction.  | [required] |
**update_transaction_tags_request** | Option<[**UpdateTransactionTagsRequest**](UpdateTransactionTagsRequest.md)> |  |  |

### Return type

 (empty response body)

### Authorization

[bearer_auth](../README.md#bearer_auth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## transactions_transaction_id_relationships_tags_post

> transactions_transaction_id_relationships_tags_post(transaction_id, update_transaction_tags_request)
Add tags to transaction

Associates one or more tags with a specific transaction. No more than 6 tags may be present on any single transaction. Duplicate tags are silently ignored. An HTTP `204` is returned on success. The associated tags, along with this request URL, are also exposed via the `tags` relationship on the transaction resource returned from `/transactions/{id}`. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**transaction_id** | **String** | The unique identifier for the transaction.  | [required] |
**update_transaction_tags_request** | Option<[**UpdateTransactionTagsRequest**](UpdateTransactionTagsRequest.md)> |  |  |

### Return type

 (empty response body)

### Authorization

[bearer_auth](../README.md#bearer_auth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

