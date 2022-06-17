#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "bindings/kv.h"

__attribute__((export_name("main"))) int main(int argc, char *argv[])
{
  // get resource descriptor
  kv_resource_descriptor_t rd;
  kv_get_kv(&rd, "my-container");

  // declare key
  kv_string_t key;
  // set key
  kv_string_set(&key, "mykey");
  // declare value
  kv_payload_t value;
  // set value
  kv_string_set(&value, "myvalue");

  // call kv.set
  kv_set(rd, &key, &value);

  // call kv.get
  kv_payload_t hostvalue;
  kv_get(rd, &key, &hostvalue);

  char *val = hostvalue.ptr;
  memcpy(&hostvalue.ptr, &val, hostvalue.len);
  printf("value from host is %s\n", val);
  kv_resource_descriptor_free(&rd);
  return 0;
}
