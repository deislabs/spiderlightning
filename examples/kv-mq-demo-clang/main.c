#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "bindings/kv.h"
#include "bindings/mq.h"

__attribute__((export_name("main"))) int main(int argc, char *argv[])
{
  kv_expected_kv_error_t kv_result;
  kv_kv_t kv;
  mq_expected_mq_error_t mq_result;
  mq_mq_t mq;
  kv_string_t kv_name;
  kv_string_set(&kv_name, "my-container");
  kv_kv_open(&kv_name, &kv_result);

  if (kv_result.is_err)
  {
    kv_error_t kv_error = kv_result.val.err;
    printf("kv_kv_open failed:  %.*s\n", (int)kv_error.val.error_with_description.len, kv_error.val.error_with_description.ptr);
    kv_error_free(&kv_error);
    exit(1);
  }
  kv = kv_result.val.ok;

  mq_string_t mq_name;
  mq_string_set(&mq_name, "spiderlightning-queue");
  mq_mq_open(&mq_name, &mq_result);

  if (mq_result.is_err)
  {
    mq_error_t mq_error = mq_result.val.err;
    printf("kv_kv_open failed:  %.*s\n", (int)mq_error.val.error_with_description.len, mq_error.val.error_with_description.ptr);
    kv_error_free(&mq_error);
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
      kv_error_free(&mq_error);
      exit(1);
    }
    mq_payload_t msg = mq_ret.val.ok;
    printf("received message: %.*s\n", (int)msg.len, msg.ptr);
    // save msg to kv
    char buf[12];
    snprintf(buf, 12, "mykey_%d", i);
    kv_string_t key;
    kv_string_set(&key, buf);
    kv_payload_t value;
    kv_string_set(&value, &msg);
    kv_expected_unit_error_t ret;
    kv_kv_set(kv, &key, &msg, &ret);
    if (ret.is_err)
    {
      kv_error_t kv_error = ret.val.err;
      printf("kv_kv_set failed:  %.*s\n", (int)kv_error.val.error_with_description.len, kv_error.val.error_with_description.ptr);
      kv_error_free(&kv_error);
      exit(1);
    }
    mq_payload_free(&msg);
  }
  for (int i = 0; i < 3; i++)
  {
    char buf[12];
    snprintf(buf, 12, "mykey_%d", i);
    kv_string_t key;
    kv_string_set(&key, buf);
    // call kv.get
    kv_payload_t hostvalue;
    kv_expected_payload_error_t ret;
    kv_kv_get(kv, &key, &ret);
    if (ret.is_err)
    {
      kv_error_t kv_error = ret.val.err;
      printf("kv_kv_get failed:  %.*s\n", (int)kv_error.val.error_with_description.len, kv_error.val.error_with_description.ptr);
      kv_error_free(&kv_error);
      exit(1);
    }
    hostvalue = ret.val.ok;

    printf("value from host is: %.*s\n", (int)hostvalue.len, hostvalue.ptr);
  }
  kv_kv_free(&kv);
  mq_mq_free(&mq);
  return 0;
}
