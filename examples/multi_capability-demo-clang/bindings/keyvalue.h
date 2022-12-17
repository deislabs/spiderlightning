#ifndef __BINDINGS_KEYVALUE_H
#define __BINDINGS_KEYVALUE_H
#ifdef __cplusplus
extern "C"
{
  #endif
  
  #include <stdint.h>
  #include <stdbool.h>
  
  typedef struct {
    uint32_t idx;
  } keyvalue_keyvalue_t;
  void keyvalue_keyvalue_free(keyvalue_keyvalue_t *ptr);
  keyvalue_keyvalue_t keyvalue_keyvalue_clone(keyvalue_keyvalue_t *ptr);
  
  typedef struct {
    char *ptr;
    size_t len;
  } keyvalue_string_t;
  
  void keyvalue_string_set(keyvalue_string_t *ret, const char *s);
  void keyvalue_string_dup(keyvalue_string_t *ret, const char *s);
  void keyvalue_string_free(keyvalue_string_t *ret);
  // common keyvalue errors
  typedef struct {
    uint8_t tag;
    union {
      keyvalue_string_t key_not_found;
      keyvalue_string_t invalid_key;
      keyvalue_string_t invalid_value;
      keyvalue_string_t connection_error;
      keyvalue_string_t authentication_error;
      keyvalue_string_t timeout_error;
      keyvalue_string_t io_error;
      keyvalue_string_t unexpected_error;
    } val;
  } keyvalue_keyvalue_error_t;
  #define KEYVALUE_KEYVALUE_ERROR_KEY_NOT_FOUND 0
  #define KEYVALUE_KEYVALUE_ERROR_INVALID_KEY 1
  #define KEYVALUE_KEYVALUE_ERROR_INVALID_VALUE 2
  #define KEYVALUE_KEYVALUE_ERROR_CONNECTION_ERROR 3
  #define KEYVALUE_KEYVALUE_ERROR_AUTHENTICATION_ERROR 4
  #define KEYVALUE_KEYVALUE_ERROR_TIMEOUT_ERROR 5
  #define KEYVALUE_KEYVALUE_ERROR_IO_ERROR 6
  #define KEYVALUE_KEYVALUE_ERROR_UNEXPECTED_ERROR 7
  void keyvalue_keyvalue_error_free(keyvalue_keyvalue_error_t *ptr);
  typedef struct {
    bool is_err;
    union {
      keyvalue_keyvalue_t ok;
      keyvalue_keyvalue_error_t err;
    } val;
  } keyvalue_expected_keyvalue_keyvalue_error_t;
  void keyvalue_expected_keyvalue_keyvalue_error_free(keyvalue_expected_keyvalue_keyvalue_error_t *ptr);
  typedef struct {
    uint8_t *ptr;
    size_t len;
  } keyvalue_list_u8_t;
  void keyvalue_list_u8_free(keyvalue_list_u8_t *ptr);
  typedef struct {
    bool is_err;
    union {
      keyvalue_list_u8_t ok;
      keyvalue_keyvalue_error_t err;
    } val;
  } keyvalue_expected_list_u8_keyvalue_error_t;
  void keyvalue_expected_list_u8_keyvalue_error_free(keyvalue_expected_list_u8_keyvalue_error_t *ptr);
  typedef struct {
    bool is_err;
    union {
      keyvalue_keyvalue_error_t err;
    } val;
  } keyvalue_expected_unit_keyvalue_error_t;
  void keyvalue_expected_unit_keyvalue_error_free(keyvalue_expected_unit_keyvalue_error_t *ptr);
  typedef struct {
    keyvalue_string_t *ptr;
    size_t len;
  } keyvalue_list_string_t;
  void keyvalue_list_string_free(keyvalue_list_string_t *ptr);
  typedef struct {
    bool is_err;
    union {
      keyvalue_list_string_t ok;
      keyvalue_keyvalue_error_t err;
    } val;
  } keyvalue_expected_list_string_keyvalue_error_t;
  void keyvalue_expected_list_string_keyvalue_error_free(keyvalue_expected_list_string_keyvalue_error_t *ptr);
  void keyvalue_keyvalue_open(keyvalue_string_t *name, keyvalue_expected_keyvalue_keyvalue_error_t *ret0);
  void keyvalue_keyvalue_get(keyvalue_keyvalue_t self, keyvalue_string_t *key, keyvalue_expected_list_u8_keyvalue_error_t *ret0);
  void keyvalue_keyvalue_set(keyvalue_keyvalue_t self, keyvalue_string_t *key, keyvalue_list_u8_t *value, keyvalue_expected_unit_keyvalue_error_t *ret0);
  void keyvalue_keyvalue_keys(keyvalue_keyvalue_t self, keyvalue_expected_list_string_keyvalue_error_t *ret0);
  void keyvalue_keyvalue_delete(keyvalue_keyvalue_t self, keyvalue_string_t *key, keyvalue_expected_unit_keyvalue_error_t *ret0);
  #ifdef __cplusplus
}
#endif
#endif
