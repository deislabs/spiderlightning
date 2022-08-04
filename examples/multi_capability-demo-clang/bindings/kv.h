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
  } kv_kv_t;
  void kv_kv_free(kv_kv_t *ptr);
  kv_kv_t kv_kv_clone(kv_kv_t *ptr);
  
  typedef struct {
    char *ptr;
    size_t len;
  } kv_string_t;
  
  void kv_string_set(kv_string_t *ret, const char *s);
  void kv_string_dup(kv_string_t *ret, const char *s);
  void kv_string_free(kv_string_t *ret);
  typedef struct {
    uint8_t tag;
    union {
      kv_string_t error_with_description;
    } val;
  } kv_error_t;
  #define KV_ERROR_ERROR_WITH_DESCRIPTION 0
  void kv_error_free(kv_error_t *ptr);
  typedef struct {
    uint8_t *ptr;
    size_t len;
  } kv_payload_t;
  void kv_payload_free(kv_payload_t *ptr);
  typedef struct {
    kv_string_t rd;
    kv_string_t key;
  } kv_observable_t;
  void kv_observable_free(kv_observable_t *ptr);
  typedef struct {
    bool is_err;
    union {
      kv_kv_t ok;
      kv_error_t err;
    } val;
  } kv_expected_kv_error_t;
  void kv_expected_kv_error_free(kv_expected_kv_error_t *ptr);
  typedef struct {
    bool is_err;
    union {
      kv_payload_t ok;
      kv_error_t err;
    } val;
  } kv_expected_payload_error_t;
  void kv_expected_payload_error_free(kv_expected_payload_error_t *ptr);
  typedef struct {
    bool is_err;
    union {
      kv_error_t err;
    } val;
  } kv_expected_unit_error_t;
  void kv_expected_unit_error_free(kv_expected_unit_error_t *ptr);
  typedef struct {
    bool is_err;
    union {
      kv_observable_t ok;
      kv_error_t err;
    } val;
  } kv_expected_observable_error_t;
  void kv_expected_observable_error_free(kv_expected_observable_error_t *ptr);
  void kv_kv_open(kv_string_t *name, kv_expected_kv_error_t *ret0);
  void kv_kv_get(kv_kv_t self, kv_string_t *key, kv_expected_payload_error_t *ret0);
  void kv_kv_set(kv_kv_t self, kv_string_t *key, kv_payload_t *value, kv_expected_unit_error_t *ret0);
  void kv_kv_delete(kv_kv_t self, kv_string_t *key, kv_expected_unit_error_t *ret0);
  void kv_kv_watch(kv_kv_t self, kv_string_t *key, kv_expected_observable_error_t *ret0);
  #ifdef __cplusplus
}
#endif
#endif
