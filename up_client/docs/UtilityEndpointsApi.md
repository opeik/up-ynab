# \UtilityEndpointsApi

All URIs are relative to *https://api.up.com.au/api/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**util_ping_get**](UtilityEndpointsApi.md#util_ping_get) | **GET** /util/ping | Ping



## util_ping_get

> crate::models::PingResponse util_ping_get()
Ping

Make a basic ping request to the API. This is useful to verify that authentication is functioning correctly. On authentication success an HTTP `200` status is returned. On failure an HTTP `401` error response is returned. 

### Parameters

This endpoint does not need any parameter.

### Return type

[**crate::models::PingResponse**](PingResponse.md)

### Authorization

[bearer_auth](../README.md#bearer_auth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

