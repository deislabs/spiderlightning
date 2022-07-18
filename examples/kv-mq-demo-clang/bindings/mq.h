#ifndef __BINDINGS_MQ_H
#define __BINDINGS_MQ_H
#ifdef __cplusplus
extern "C"
{
  #endif
  
  #include <stdint.h>
  #include <stdbool.h>
  
  typedef struct {
    uint32_t idx;
  } mq_mq_t;
  void mq_mq_free(mq_mq_t *ptr);
  mq_mq_t mq_mq_clone(mq_mq_t *ptr);
  
  typedef struct {
    char *ptr;
    size_t len;
  } mq_string_t;
  
  void mq_string_set(mq_string_t *ret, const char *s);
  void mq_string_dup(mq_string_t *ret, const char *s);
  void mq_string_free(mq_string_t *ret);
  typedef struct {
    uint8_t tag;
    union {
      mq_string_t error_with_description;
    } val;
  } mq_error_t;
  #define MQ_ERROR_ERROR_WITH_DESCRIPTION 0
  void mq_error_free(mq_error_t *ptr);
  typedef struct {
    uint8_t *ptr;
    size_t len;
  } mq_payload_t;
  void mq_payload_free(mq_payload_t *ptr);
  typedef struct {
    mq_string_t rd;
    mq_string_t key;
  } mq_observable_t;
  void mq_observable_free(mq_observable_t *ptr);
  typedef struct {
    bool is_err;
    union {
      mq_mq_t ok;
      mq_error_t err;
    } val;
  } mq_expected_mq_error_t;
  void mq_expected_mq_error_free(mq_expected_mq_error_t *ptr);
  typedef struct {
    bool is_err;
    union {
      mq_error_t err;
    } val;
  } mq_expected_unit_error_t;
  void mq_expected_unit_error_free(mq_expected_unit_error_t *ptr);
  typedef struct {
    bool is_err;
    union {
      mq_payload_t ok;
      mq_error_t err;
    } val;
  } mq_expected_payload_error_t;
  void mq_expected_payload_error_free(mq_expected_payload_error_t *ptr);
  void mq_mq_open(mq_string_t *name, mq_expected_mq_error_t *ret0);
  void mq_mq_send(mq_mq_t self, mq_payload_t *msg, mq_expected_unit_error_t *ret0);
  void mq_mq_receive(mq_mq_t self, mq_expected_payload_error_t *ret0);
  #ifdef __cplusplus
}
#endif
#endif
