# ErrorObject

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**status** | **String** | The HTTP status code associated with this error. This can also be obtained from the response headers. The status indicates the broad type of error according to HTTP semantics.  | 
**title** | **String** | A short description of this error. This should be stable across multiple occurrences of this type of error and typically expands on the reason for the status code.  | 
**detail** | **String** | A detailed description of this error. This should be considered unique to individual occurrences of an error and subject to change. It is useful for debugging purposes.  | 
**source** | Option<[**crate::models::ErrorObjectSource**](ErrorObject_source.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


