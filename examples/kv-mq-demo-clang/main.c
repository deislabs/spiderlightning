#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "bindings/kv.h"
#include "bindings/mq.h"

__attribute__((export_name("main"))) int main(int argc, char *argv[])
{
  // get resource descriptor
  kv_resource_descriptor_t kv;
  mq_resource_descriptor_t mq;
  kv_string_t kv_name;
  kv_string_set(&kv_name, "my-container");
  kv_get_kv(&kv_name, &kv);

  mq_string_t mq_name;
  mq_string_set(&mq_name, "wasi-cloud-queue");
  mq_get_mq(&mq_name, &mq);

  for (int i = 0; i < 3; i++)
  {
    mq_payload_t msg;
    mq_receive(&mq, &msg);
    printf("received message: %.*s\n", (int)msg.len, msg.ptr);
    // save msg to kv
    char buf[12];
    snprintf(buf, 12, "mykey_%d", i);
    kv_string_t key;
    kv_string_set(&key, buf);
    kv_payload_t value;
    kv_string_set(&value, &msg);
    kv_set(&kv, &key, &msg);
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
    kv_get(&kv, &key, &hostvalue);

    printf("value from host is: %.*s\n", (int)hostvalue.len, hostvalue.ptr);
  }
  kv_resource_descriptor_free(&kv);
  mq_resource_descriptor_free(&mq);
  return 0;
}
