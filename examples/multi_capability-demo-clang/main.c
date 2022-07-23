#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "bindings/state-store.h"
#include "bindings/mq.h"

__attribute__((export_name("main"))) int main(int argc, char *argv[])
{
  state_store_expected_state_store_error_t state_store_result;
  state_store_state_store_t state_store;
  mq_expected_mq_error_t mq_result;
  mq_mq_t mq;
  state_store_string_t state_store_name;
  state_store_string_set(&state_store_name, "my-container");
  state_store_state_store_open(&state_store_name, &state_store_result);

  if (state_store_result.is_err)
  {
    state_store_error_t state_store_error = state_store_result.val.err;
    printf("state_store_state_store_open failed:  %.*s\n", (int)state_store_error.val.error_with_description.len, state_store_error.val.error_with_description.ptr);
    state_store_error_free(&state_store_error);
    exit(1);
  }
  state_store = state_store_result.val.ok;

  mq_string_t mq_name;
  mq_string_set(&mq_name, "spiderlightning-queue");
  mq_mq_open(&mq_name, &mq_result);

  if (mq_result.is_err)
  {
    mq_error_t mq_error = mq_result.val.err;
    printf("state_store_state_store_open failed:  %.*s\n", (int)mq_error.val.error_with_description.len, mq_error.val.error_with_description.ptr);
    state_store_error_free(&mq_error);
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
      state_store_error_free(&mq_error);
      exit(1);
    }
    mq_payload_t msg = mq_ret.val.ok;
    printf("received message: %.*s\n", (int)msg.len, msg.ptr);
    // save msg to state_store
    char buf[12];
    snprintf(buf, 12, "mykey_%d", i);
    state_store_string_t key;
    state_store_string_set(&key, buf);
    state_store_payload_t value;
    state_store_string_set(&value, &msg);
    state_store_expected_unit_error_t ret;
    state_store_state_store_set(state_store, &key, &msg, &ret);
    if (ret.is_err)
    {
      state_store_error_t state_store_error = ret.val.err;
      printf("state_store_state_store_set failed:  %.*s\n", (int)state_store_error.val.error_with_description.len, state_store_error.val.error_with_description.ptr);
      state_store_error_free(&state_store_error);
      exit(1);
    }
    mq_payload_free(&msg);
  }
  for (int i = 0; i < 3; i++)
  {
    char buf[12];
    snprintf(buf, 12, "mykey_%d", i);
    state_store_string_t key;
    state_store_string_set(&key, buf);
    // call state_store.get
    state_store_payload_t hostvalue;
    state_store_expected_payload_error_t ret;
    state_store_state_store_get(state_store, &key, &ret);
    if (ret.is_err)
    {
      state_store_error_t state_store_error = ret.val.err;
      printf("state_store_state_store_get failed:  %.*s\n", (int)state_store_error.val.error_with_description.len, state_store_error.val.error_with_description.ptr);
      state_store_error_free(&state_store_error);
      exit(1);
    }
    hostvalue = ret.val.ok;

    printf("value from host is: %.*s\n", (int)hostvalue.len, hostvalue.ptr);
  }
  state_store_state_store_free(&state_store);
  mq_mq_free(&mq);
  return 0;
}
