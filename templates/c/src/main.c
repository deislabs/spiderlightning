#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "bindings/kv.h"

__attribute__((export_name("main"))) int main(int argc, char *argv[])
{
    // open kv and get kv handle
    kv_expected_kv_error_t open_result;
    kv_kv_t kv;
    kv_string_t kv_name;
    kv_string_set(&kv_name, "my-folder");
    kv_kv_open(&kv_name, &open_result);
    if (open_result.is_err)
    {
        kv_error_t kv_error = open_result.val.err;
        printf("kv_kv_open failed:  %.*s\n", (int)kv_error.val.error_with_description.len, kv_error.val.error_with_description.ptr);
        kv_error_free(&kv_error);
        exit(1);
    }
    kv = open_result.val.ok;

    // set key-value pair
    kv_expected_unit_error_t set_result;
    kv_string_t key;
    kv_string_set(&key, "hello-spiderlightning");
    const char * payload_ptr = "Hello, SpiderLightning!";
    kv_payload_t payload = {
        .ptr = (uint8_t *) payload_ptr,
        .len = strlen(payload_ptr)
    };
    kv_kv_set(kv, &key, &payload, &set_result);
    if (set_result.is_err)
    {
        kv_error_t kv_error = set_result.val.err;
        printf("kv_kv_set failed:  %.*s\n", (int)kv_error.val.error_with_description.len, kv_error.val.error_with_description.ptr);
        kv_error_free(&kv_error);
        exit(1);
    }

    // get key-value pair
    kv_payload_t get_value;
    kv_expected_payload_error_t get_result;
    kv_kv_get(kv, &key, &get_result);
    if (get_result.is_err)
    {
      kv_error_t kv_error = get_result.val.err;
      printf("kv_kv_get failed:  %.*s\n", (int)kv_error.val.error_with_description.len, kv_error.val.error_with_description.ptr);
      kv_error_free(&kv_error);
      exit(1);
    }
    get_value = get_result.val.ok;
    printf("%.*s\n", (int)get_value.len, get_value.ptr);
    
}