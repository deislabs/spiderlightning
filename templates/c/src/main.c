#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "bindings/keyvalue.h"

__attribute__((export_name("main"))) int main(int argc, char *argv[])
{
    // open keyvalue and get keyvalue handle
    keyvalue_expected_keyvalue_error_t open_result;
    keyvalue_keyvalue_t keyvalue;
    keyvalue_string_t keyvalue_name;
    keyvalue_string_set(&keyvalue_name, "placeholder-name");
    keyvalue_keyvalue_open(&keyvalue_name, &open_result);
    if (open_result.is_err)
    {
        keyvalue_error_t keyvalue_error = open_result.val.err;
        printf("keyvalue_keyvalue_open failed:  %.*s\n", (int)keyvalue_error.val.error_with_description.len, keyvalue_error.val.error_with_description.ptr);
        keyvalue_error_free(&keyvalue_error);
        exit(1);
    }
    keyvalue = open_result.val.ok;

    // set key-value pair
    keyvalue_expected_unit_error_t set_result;
    keyvalue_string_t key;
    keyvalue_string_set(&key, "hello-spiderlightning");
    const char * payload_ptr = "Hello, SpiderLightning!";
    keyvalue_payload_t payload = {
        .ptr = (uint8_t *) payload_ptr,
        .len = strlen(payload_ptr)
    };
    keyvalue_keyvalue_set(keyvalue, &key, &payload, &set_result);
    if (set_result.is_err)
    {
        keyvalue_error_t keyvalue_error = set_result.val.err;
        printf("keyvalue_keyvalue_set failed:  %.*s\n", (int)keyvalue_error.val.error_with_description.len, keyvalue_error.val.error_with_description.ptr);
        keyvalue_error_free(&keyvalue_error);
        exit(1);
    }

    // get key-value pair
    keyvalue_payload_t get_value;
    keyvalue_expected_payload_error_t get_result;
    keyvalue_keyvalue_get(keyvalue, &key, &get_result);
    if (get_result.is_err)
    {
      keyvalue_error_t keyvalue_error = get_result.val.err;
      printf("keyvalue_keyvalue_get failed:  %.*s\n", (int)keyvalue_error.val.error_with_description.len, keyvalue_error.val.error_with_description.ptr);
      keyvalue_error_free(&keyvalue_error);
      exit(1);
    }
    get_value = get_result.val.ok;
    printf("%.*s\n", (int)get_value.len, get_value.ptr);
    
}