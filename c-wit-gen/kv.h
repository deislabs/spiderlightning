#ifndef __BINDINGS_KV_H
#define __BINDINGS_KV_H
#ifdef __cplusplus
extern "C"
{
  #endif
  
  #include <stdint.h>
  #include <stdbool.h>
  #include <stddef.h>
  typedef struct {
    uint32_t idx;
  } blob_resource_descriptor_t;
  void blob_resource_descriptor_free(blob_resource_descriptor_t *ptr);
  blob_resource_descriptor_t blob_resource_descriptor_clone(blob_resource_descriptor_t *ptr);
  
  typedef struct {
    char *ptr;
    size_t len;
  } blob_string_t;
  
  void blob_string_set(blob_string_t *ret, const char *s);
  void blob_string_dup(blob_string_t *ret, const char *s);
  void blob_string_free(blob_string_t *ret);
  typedef uint8_t blob_error_t;
  #define KV_ERROR_SUCCESS 0
  #define KV_ERROR_ERROR 1
  typedef struct {
    uint8_t *ptr;
    size_t len;
  } blob_payload_t;
  void blob_payload_free(blob_payload_t *ptr);
  blob_error_t blob_get_blob(blob_resource_descriptor_t *ret0);
  blob_error_t blob_get(blob_resource_descriptor_t rd, blob_string_t *key, blob_payload_t *ret0);
  blob_error_t blob_set(blob_resource_descriptor_t rd, blob_string_t *key, blob_payload_t *value);
  blob_error_t blob_delete(blob_resource_descriptor_t rd, blob_string_t *key);
  #ifdef __cplusplus
}
#endif
#endif
