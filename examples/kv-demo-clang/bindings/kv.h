#ifndef __BINDINGS_KV_H
#define __BINDINGS_KV_H
#ifdef __cplusplus
extern "C"
{
  #endif
  
  #include <stdint.h>
  #include <stdbool.h>
  
  typedef struct {
    uint32_t idx;
  } kv_resource_descriptor_t;
  void kv_resource_descriptor_free(kv_resource_descriptor_t *ptr);
  kv_resource_descriptor_t kv_resource_descriptor_clone(kv_resource_descriptor_t *ptr);
  
  typedef struct {
    char *ptr;
    size_t len;
  } kv_string_t;
  
  void kv_string_set(kv_string_t *ret, const char *s);
  void kv_string_dup(kv_string_t *ret, const char *s);
  void kv_string_free(kv_string_t *ret);
  typedef uint8_t kv_error_t;
  #define KV_ERROR_DESCRIPTOR_ERROR 0
  #define KV_ERROR_IO_ERROR 1
  #define KV_ERROR_OTHER_ERROR 2
  typedef struct {
    uint8_t *ptr;
    size_t len;
  } kv_payload_t;
  void kv_payload_free(kv_payload_t *ptr);
  kv_error_t kv_get_kv(kv_resource_descriptor_t *ret0);
  kv_error_t kv_get(kv_resource_descriptor_t rd, kv_string_t *key, kv_payload_t *ret0);
  kv_error_t kv_set(kv_resource_descriptor_t rd, kv_string_t *key, kv_payload_t *value);
  kv_error_t kv_delete(kv_resource_descriptor_t rd, kv_string_t *key);
  #ifdef __cplusplus
}
#endif
#endif
