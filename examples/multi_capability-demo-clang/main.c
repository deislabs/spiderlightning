#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "bindings/keyvalue.h"
#include "bindings/mq.h"

__attribute__((export_name("main"))) int main(int argc, char *argv[])
{
  keyvalue_expected_keyvalue_error_t keyvalue_result;
  keyvalue_keyvalue_t keyvalue;
  mq_expected_mq_error_t mq_result;
  mq_mq_t mq;
  keyvalue_string_t keyvalue_name;
  keyvalue_string_set(&keyvalue_name, "my-container");
  keyvalue_keyvalue_open(&keyvalue_name, &keyvalue_result);

  if (keyvalue_result.is_err)
  {
    keyvalue_error_t keyvalue_error = keyvalue_result.val.err;
    printf("keyvalue_keyvalue_open failed:  %.*s\n", (int)keyvalue_error.val.error_with_description.len, keyvalue_error.val.error_with_description.ptr);
    keyvalue_error_free(&keyvalue_error);
    exit(1);
  }
  keyvalue = keyvalue_result.val.ok;

  mq_string_t mq_name;
  mq_string_set(&mq_name, "wasi-cloud-queue");
  mq_mq_open(&mq_name, &mq_result);

  if (mq_result.is_err)
  {
    mq_error_t mq_error = mq_result.val.err;
    printf("keyvalue_keyvalue_open failed:  %.*s\n", (int)mq_error.val.error_with_description.len, mq_error.val.error_with_description.ptr);
    mq_error_free(&mq_error);
    exit(1);
  }
  mq = mq_result.val.ok;

  for (int i = 0; i < 3; i++)
  {
    mq_expected_payload_error_t mq_ret;
    mq_mq_receive(mq, &mq_ret);
    if (mq_ret.is_err)
    {
      mq_error_t mq_error = mq_ret.val.err;
      printf("mq_mq_receive failed:  %.*s\n", (int)mq_error.val.error_with_description.len, mq_error.val.error_with_description.ptr);
      mq_error_free(&mq_error);
      exit(1);
    }
    mq_payload_t msg = mq_ret.val.ok;
    printf("received message: %.*s\n", (int)msg.len, msg.ptr);
    // save msg to keyvalue
    char buf[12];
    snprintf(buf, 12, "mykey_%d", i);
    keyvalue_string_t key;
    keyvalue_string_set(&key, buf);
    keyvalue_expected_unit_error_t ret;
    keyvalue_payload_t payload = {
      .ptr = msg.ptr,
      .len = msg.len
    };
    keyvalue_keyvalue_set(keyvalue, &key, &payload, &ret);
    if (ret.is_err)
    {
      keyvalue_error_t keyvalue_error = ret.val.err;
      printf("keyvalue_keyvalue_set failed:  %.*s\n", (int)keyvalue_error.val.error_with_description.len, keyvalue_error.val.error_with_description.ptr);
      keyvalue_error_free(&keyvalue_error);
      exit(1);
    }
    mq_payload_free(&msg);
  }
  for (int i = 0; i < 3; i++)
  {
    char buf[12];
    snprintf(buf, 12, "mykey_%d", i);
    keyvalue_string_t key;
    keyvalue_string_set(&key, buf);
    // call keyvalue.get
    keyvalue_payload_t hostvalue;
    keyvalue_expected_payload_error_t ret;
    keyvalue_keyvalue_get(keyvalue, &key, &ret);
    if (ret.is_err)
    {
      keyvalue_error_t keyvalue_error = ret.val.err;
      printf("keyvalue_keyvalue_get failed:  %.*s\n", (int)keyvalue_error.val.error_with_description.len, keyvalue_error.val.error_with_description.ptr);
      keyvalue_error_free(&keyvalue_error);
      exit(1);
    }
    hostvalue = ret.val.ok;

    printf("value from host is: %.*s\n", (int)hostvalue.len, hostvalue.ptr);
  }
  keyvalue_keyvalue_free(&keyvalue);
  mq_mq_free(&mq);
  return 0;
}
