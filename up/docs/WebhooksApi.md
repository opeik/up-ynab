# \WebhooksApi

All URIs are relative to *https://api.up.com.au/api/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**webhooks_get**](WebhooksApi.md#webhooks_get) | **GET** /webhooks | List webhooks
[**webhooks_id_delete**](WebhooksApi.md#webhooks_id_delete) | **DELETE** /webhooks/{id} | Delete webhook
[**webhooks_id_get**](WebhooksApi.md#webhooks_id_get) | **GET** /webhooks/{id} | Retrieve webhook
[**webhooks_post**](WebhooksApi.md#webhooks_post) | **POST** /webhooks | Create webhook
[**webhooks_webhook_id_logs_get**](WebhooksApi.md#webhooks_webhook_id_logs_get) | **GET** /webhooks/{webhookId}/logs | List webhook logs
[**webhooks_webhook_id_ping_post**](WebhooksApi.md#webhooks_webhook_id_ping_post) | **POST** /webhooks/{webhookId}/ping | Ping webhook



## webhooks_get

> crate::models::ListWebhooksResponse webhooks_get(page_left_square_bracket_size_right_square_bracket)
List webhooks

Retrieve a list of configured webhooks. The returned list is [paginated](#pagination) and can be scrolled by following the `next` and `prev` links where present. Results are ordered oldest first to newest last. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**page_left_square_bracket_size_right_square_bracket** | Option<**i32**> | The number of records to return in each page.  |  |

### Return type

[**crate::models::ListWebhooksResponse**](ListWebhooksResponse.md)

### Authorization

[bearer_auth](../README.md#bearer_auth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## webhooks_id_delete

> webhooks_id_delete(id)
Delete webhook

Delete a specific webhook by providing its unique identifier. Once deleted, webhook events will no longer be sent to the configured URL. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | The unique identifier for the webhook.  | [required] |

### Return type

 (empty response body)

### Authorization

[bearer_auth](../README.md#bearer_auth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## webhooks_id_get

> crate::models::GetWebhookResponse webhooks_id_get(id)
Retrieve webhook

Retrieve a specific webhook by providing its unique identifier. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | The unique identifier for the webhook.  | [required] |

### Return type

[**crate::models::GetWebhookResponse**](GetWebhookResponse.md)

### Authorization

[bearer_auth](../README.md#bearer_auth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## webhooks_post

> crate::models::CreateWebhookResponse webhooks_post(create_webhook_request)
Create webhook

Create a new webhook with a given URL. The URL will receive webhook events as JSON-encoded `POST` requests. The URL must respond with a HTTP `200` status on success.  There is currently a limit of 10 webhooks at any given time. Once this limit is reached, existing webhooks will need to be deleted before new webhooks can be created.  Event delivery is retried with exponential backoff if the URL is unreachable or it does not respond with a `200` status. The response includes a `secretKey` attribute, which is used to sign requests sent to the webhook URL. It will not be returned from any other endpoints within the Up API. If the `secretKey` is lost, simply create a new webhook with the same URL, capture its `secretKey` and then delete the original webhook. See [Handling webhook events](#callback_post_webhookURL) for details on how to process webhook events.  It is probably a good idea to test the webhook by [sending it a `PING` event](#post_webhooks_webhookId_ping) after creating it. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_webhook_request** | Option<[**CreateWebhookRequest**](CreateWebhookRequest.md)> |  |  |

### Return type

[**crate::models::CreateWebhookResponse**](CreateWebhookResponse.md)

### Authorization

[bearer_auth](../README.md#bearer_auth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## webhooks_webhook_id_logs_get

> crate::models::ListWebhookDeliveryLogsResponse webhooks_webhook_id_logs_get(webhook_id, page_left_square_bracket_size_right_square_bracket)
List webhook logs

Retrieve a list of delivery logs for a webhook by providing its unique identifier. This is useful for analysis and debugging purposes. The returned list is [paginated](#pagination) and can be scrolled by following the `next` and `prev` links where present. Results are ordered newest first to oldest last. Logs may be automatically purged after a period of time. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**webhook_id** | **String** | The unique identifier for the webhook.  | [required] |
**page_left_square_bracket_size_right_square_bracket** | Option<**i32**> | The number of records to return in each page.  |  |

### Return type

[**crate::models::ListWebhookDeliveryLogsResponse**](ListWebhookDeliveryLogsResponse.md)

### Authorization

[bearer_auth](../README.md#bearer_auth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## webhooks_webhook_id_ping_post

> crate::models::WebhookEventCallback webhooks_webhook_id_ping_post(webhook_id)
Ping webhook

Send a `PING` event to a webhook by providing its unique identifier. This is useful for testing and debugging purposes. The event is delivered asynchronously and its data is returned in the response to this request. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**webhook_id** | **String** | The unique identifier for the webhook.  | [required] |

### Return type

[**crate::models::WebhookEventCallback**](WebhookEventCallback.md)

### Authorization

[bearer_auth](../README.md#bearer_auth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

