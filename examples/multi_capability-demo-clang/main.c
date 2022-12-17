#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "bindings/keyvalue.h"
#include "bindings/messaging.h"

__attribute__((export_name("main"))) int main(int argc, char *argv[])
{
  // <opening keyvalue capability>
  keyvalue_expected_keyvalue_keyvalue_error_t keyvalue_result;
  keyvalue_keyvalue_t keyvalue;
  keyvalue_string_t keyvalue_name;
  keyvalue_string_set(&keyvalue_name, "my-container");
  keyvalue_keyvalue_open(&keyvalue_name, &keyvalue_result);

  if (keyvalue_result.is_err)
  {
    keyvalue_keyvalue_error_t keyvalue_error = keyvalue_result.val.err;
    printf("keyvalue_keyvalue_open failed:  %.*s\n", (int)keyvalue_error.val.unexpected_error.len, keyvalue_error.val.unexpected_error.ptr);
    keyvalue_keyvalue_error_free(&keyvalue_error);
    exit(1);
  }
  keyvalue = keyvalue_result.val.ok;
  // </>

  // <opening messaging capability>
  messaging_expected_sub_messaging_error_t messaging_result;
  messaging_sub_t messaging;
  messaging_string_t messaging_name;
  messaging_string_set(&messaging_name, "my-messaging");
  messaging_sub_open(&messaging_name, &messaging_result);

  if (messaging_result.is_err)
  {
    messaging_messaging_error_t messaging_error = messaging_result.val.err;
    printf("keyvalue_keyvalue_open failed:  %.*s\n", (int)messaging_error.val.unexpected_error.len, messaging_error.val.unexpected_error.ptr);
    messaging_messaging_error_free(&messaging_error);
    exit(1);
  }
  messaging = messaging_result.val.ok;
  // </>

  // <make a messaging subscription>
  messaging_expected_subscription_token_messaging_error_t sub_tok_result;
  messaging_subscription_token_t sub_tok;
  messaging_string_t topic;
  messaging_string_set(&topic, "rust");
  messaging_sub_subscribe(messaging, &topic, &sub_tok_result);

  if (sub_tok_result.is_err)
  {
    messaging_messaging_error_t sub_messaging_error = sub_tok_result.val.err;
    printf("keyvalue_keyvalue_open failed:  %.*s\n", (int)sub_messaging_error.val.unexpected_error.len, sub_messaging_error.val.unexpected_error.ptr);
    messaging_messaging_error_free(&sub_messaging_error);
    exit(1);
  }
  sub_tok = sub_tok_result.val.ok;
  // </>

  for (int i = 0; i < 3; i++)
  {
    // <receive a message>
    messaging_expected_list_u8_messaging_error_t messaging_ret;
    messaging_sub_receive(messaging, &sub_tok, &messaging_ret);
    if (messaging_ret.is_err)
    {
      messaging_messaging_error_t messaging_error = messaging_ret.val.err;
      printf("messaging_sub_receive failed:  %.*s\n", (int)messaging_error.val.unexpected_error.len, messaging_error.val.unexpected_error.ptr);
      messaging_messaging_error_free(&messaging_error);
      exit(1);
    }
    messaging_list_u8_t msg = messaging_ret.val.ok;
    printf("received message: %.*s\n", (int)msg.len, msg.ptr);
    // </>

    // <save msg to keyvalue>
    char buf[12];
    snprintf(buf, 12, "mykey_%d", i);
    keyvalue_string_t key;
    keyvalue_string_set(&key, buf);
    keyvalue_expected_unit_keyvalue_error_t ret;
    keyvalue_list_u8_t payload = {
      .ptr = msg.ptr,
      .len = msg.len
    };
    keyvalue_keyvalue_set(keyvalue, &key, &payload, &ret);
    if (ret.is_err)
    {
      keyvalue_keyvalue_error_t keyvalue_error = ret.val.err;
      printf("keyvalue_keyvalue_set failed:  %.*s\n", (int)keyvalue_error.val.unexpected_error.len, keyvalue_error.val.unexpected_error.ptr);
      keyvalue_keyvalue_error_free(&keyvalue_error);
      exit(1);
    }
    // </>

    messaging_list_u8_free(&msg);
  }

  for (int i = 0; i < 3; i++)
  {
    char buf[12];
    snprintf(buf, 12, "mykey_%d", i);
    keyvalue_string_t key;
    keyvalue_string_set(&key, buf);
    // <get msg from keyvalue>
    keyvalue_list_u8_t hostvalue;
    keyvalue_expected_list_u8_keyvalue_error_t ret;
    keyvalue_keyvalue_get(keyvalue, &key, &ret);
    if (ret.is_err)
    {
      keyvalue_keyvalue_error_t keyvalue_error = ret.val.err;
      printf("keyvalue_keyvalue_get failed:  %.*s\n", (int)keyvalue_error.val.unexpected_error.len, keyvalue_error.val.unexpected_error.ptr);
      keyvalue_keyvalue_error_free(&keyvalue_error);
      exit(1);
    }
    hostvalue = ret.val.ok;
    printf("value from host is: %.*s\n", (int)hostvalue.len, hostvalue.ptr);
    // </>
  }
  
  keyvalue_keyvalue_free(&keyvalue);
  messaging_sub_free(&messaging);
  return 0;
}
