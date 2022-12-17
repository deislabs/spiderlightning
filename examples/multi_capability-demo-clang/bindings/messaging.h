#ifndef __BINDINGS_MESSAGING_H
#define __BINDINGS_MESSAGING_H
#ifdef __cplusplus
extern "C"
{
  #endif
  
  #include <stdint.h>
  #include <stdbool.h>
  
  typedef struct {
    uint32_t idx;
  } messaging_pub_t;
  void messaging_pub_free(messaging_pub_t *ptr);
  messaging_pub_t messaging_pub_clone(messaging_pub_t *ptr);
  
  typedef struct {
    uint32_t idx;
  } messaging_sub_t;
  void messaging_sub_free(messaging_sub_t *ptr);
  messaging_sub_t messaging_sub_clone(messaging_sub_t *ptr);
  
  typedef struct {
    char *ptr;
    size_t len;
  } messaging_string_t;
  
  void messaging_string_set(messaging_string_t *ret, const char *s);
  void messaging_string_dup(messaging_string_t *ret, const char *s);
  void messaging_string_free(messaging_string_t *ret);
  // a messaging interface
  // common messaging errors
  typedef struct {
    uint8_t tag;
    union {
      messaging_string_t payload_too_large;
      messaging_string_t queue_or_topic_not_found;
      messaging_string_t insufficient_permissions;
      messaging_string_t service_unavailable;
      messaging_string_t delivery_failed;
      messaging_string_t connection_lost;
      messaging_string_t unsupported_message_format;
      messaging_string_t unexpected_error;
    } val;
  } messaging_messaging_error_t;
  #define MESSAGING_MESSAGING_ERROR_PAYLOAD_TOO_LARGE 0
  #define MESSAGING_MESSAGING_ERROR_QUEUE_OR_TOPIC_NOT_FOUND 1
  #define MESSAGING_MESSAGING_ERROR_INSUFFICIENT_PERMISSIONS 2
  #define MESSAGING_MESSAGING_ERROR_SERVICE_UNAVAILABLE 3
  #define MESSAGING_MESSAGING_ERROR_DELIVERY_FAILED 4
  #define MESSAGING_MESSAGING_ERROR_CONNECTION_LOST 5
  #define MESSAGING_MESSAGING_ERROR_UNSUPPORTED_MESSAGE_FORMAT 6
  #define MESSAGING_MESSAGING_ERROR_UNEXPECTED_ERROR 7
  void messaging_messaging_error_free(messaging_messaging_error_t *ptr);
  // provides a handle to a consumer that owns a specific subscription
  typedef messaging_string_t messaging_subscription_token_t;
  void messaging_subscription_token_free(messaging_subscription_token_t *ptr);
  typedef struct {
    bool is_err;
    union {
      messaging_pub_t ok;
      messaging_messaging_error_t err;
    } val;
  } messaging_expected_pub_messaging_error_t;
  void messaging_expected_pub_messaging_error_free(messaging_expected_pub_messaging_error_t *ptr);
  typedef struct {
    uint8_t *ptr;
    size_t len;
  } messaging_list_u8_t;
  void messaging_list_u8_free(messaging_list_u8_t *ptr);
  typedef struct {
    bool is_err;
    union {
      messaging_messaging_error_t err;
    } val;
  } messaging_expected_unit_messaging_error_t;
  void messaging_expected_unit_messaging_error_free(messaging_expected_unit_messaging_error_t *ptr);
  typedef struct {
    bool is_err;
    union {
      messaging_sub_t ok;
      messaging_messaging_error_t err;
    } val;
  } messaging_expected_sub_messaging_error_t;
  void messaging_expected_sub_messaging_error_free(messaging_expected_sub_messaging_error_t *ptr);
  typedef struct {
    bool is_err;
    union {
      messaging_subscription_token_t ok;
      messaging_messaging_error_t err;
    } val;
  } messaging_expected_subscription_token_messaging_error_t;
  void messaging_expected_subscription_token_messaging_error_free(messaging_expected_subscription_token_messaging_error_t *ptr);
  typedef struct {
    bool is_err;
    union {
      messaging_list_u8_t ok;
      messaging_messaging_error_t err;
    } val;
  } messaging_expected_list_u8_messaging_error_t;
  void messaging_expected_list_u8_messaging_error_free(messaging_expected_list_u8_messaging_error_t *ptr);
  void messaging_pub_open(messaging_string_t *name, messaging_expected_pub_messaging_error_t *ret0);
  void messaging_pub_publish(messaging_pub_t self, messaging_list_u8_t *msg, messaging_string_t *topic, messaging_expected_unit_messaging_error_t *ret0);
  void messaging_sub_open(messaging_string_t *name, messaging_expected_sub_messaging_error_t *ret0);
  void messaging_sub_subscribe(messaging_sub_t self, messaging_string_t *topic, messaging_expected_subscription_token_messaging_error_t *ret0);
  void messaging_sub_receive(messaging_sub_t self, messaging_subscription_token_t *sub_tok, messaging_expected_list_u8_messaging_error_t *ret0);
  #ifdef __cplusplus
}
#endif
#endif
