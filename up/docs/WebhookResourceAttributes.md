# WebhookResourceAttributes

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**url** | **String** | The URL that this webhook is configured to `POST` events to.  | 
**description** | Option<**String**> | An optional description that was provided at the time the webhook was created.  | 
**secret_key** | Option<**String**> | A shared secret key used to sign all webhook events sent to the configured webhook URL. This field is returned only once, upon the initial creation of the webhook. If lost, create a new webhook and delete this webhook.  The webhook URL receives a request with a `X-Up-Authenticity-Signature` header, which is the SHA-256 HMAC of the entire raw request body signed using this `secretKey`. It is advised to compute and check this signature to verify the authenticity of requests sent to the webhook URL. See [Handling webhook events](#callback_post_webhookURL) for full details.  | [optional]
**created_at** | **String** | The date-time at which this webhook was created.  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


