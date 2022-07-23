#ifndef __BINDINGS_STATE_STORE_H
#define __BINDINGS_STATE_STORE_H
#ifdef __cplusplus
extern "C"
{
  #endif
  
  #include <stdint.h>
  #include <stdbool.h>
  
  typedef struct {
    uint32_t idx;
  } state_store_state_store_t;
  void state_store_state_store_free(state_store_state_store_t *ptr);
  state_store_state_store_t state_store_state_store_clone(state_store_state_store_t *ptr);
  
  typedef struct {
    char *ptr;
    size_t len;
  } state_store_string_t;
  
  void state_store_string_set(state_store_string_t *ret, const char *s);
  void state_store_string_dup(state_store_string_t *ret, const char *s);
  void state_store_string_free(state_store_string_t *ret);
  typedef struct {
    uint8_t tag;
    union {
      state_store_string_t error_with_description;
    } val;
  } state_store_error_t;
  #define STATE_STORE_ERROR_ERROR_WITH_DESCRIPTION 0
  void state_store_error_free(state_store_error_t *ptr);
  typedef struct {
    uint8_t *ptr;
    size_t len;
  } state_store_payload_t;
  void state_store_payload_free(state_store_payload_t *ptr);
  typedef struct {
    state_store_string_t rd;
    state_store_string_t key;
  } state_store_observable_t;
  void state_store_observable_free(state_store_observable_t *ptr);
  typedef struct {
    bool is_err;
    union {
      state_store_state_store_t ok;
      state_store_error_t err;
    } val;
  } state_store_expected_state_store_error_t;
  void state_store_expected_state_store_error_free(state_store_expected_state_store_error_t *ptr);
  typedef struct {
    bool is_err;
    union {
      state_store_payload_t ok;
      state_store_error_t err;
    } val;
  } state_store_expected_payload_error_t;
  void state_store_expected_payload_error_free(state_store_expected_payload_error_t *ptr);
  typedef struct {
    bool is_err;
    union {
      state_store_error_t err;
    } val;
  } state_store_expected_unit_error_t;
  void state_store_expected_unit_error_free(state_store_expected_unit_error_t *ptr);
  typedef struct {
    bool is_err;
    union {
      state_store_observable_t ok;
      state_store_error_t err;
    } val;
  } state_store_expected_observable_error_t;
  void state_store_expected_observable_error_free(state_store_expected_observable_error_t *ptr);
  void state_store_state_store_open(state_store_string_t *name, state_store_expected_state_store_error_t *ret0);
  void state_store_state_store_get(state_store_state_store_t self, state_store_string_t *key, state_store_expected_payload_error_t *ret0);
  void state_store_state_store_set(state_store_state_store_t self, state_store_string_t *key, state_store_payload_t *value, state_store_expected_unit_error_t *ret0);
  void state_store_state_store_delete(state_store_state_store_t self, state_store_string_t *key, state_store_expected_unit_error_t *ret0);
  void state_store_state_store_watch(state_store_state_store_t self, state_store_string_t *key, state_store_expected_observable_error_t *ret0);
  #ifdef __cplusplus
}
#endif
#endif
