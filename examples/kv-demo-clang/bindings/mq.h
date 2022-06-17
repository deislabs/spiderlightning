#ifndef __BINDINGS_MQ_H
#define __BINDINGS_MQ_H
#ifdef __cplusplus
extern "C"
{
  #endif
  
  #include <stdint.h>
  #include <stdbool.h>
  
  typedef struct {
    char *ptr;
    size_t len;
  } mq_string_t;
  
  void mq_string_set(mq_string_t *ret, const char *s);
  void mq_string_dup(mq_string_t *ret, const char *s);
  void mq_string_free(mq_string_t *ret);
  typedef uint8_t mq_error_t;
  #define MQ_ERROR_DESCRIPTOR_ERROR 0
  #define MQ_ERROR_IO_ERROR 1
  #define MQ_ERROR_OTHER_ERROR 2
  typedef struct {
    uint8_t *ptr;
    size_t len;
  } mq_payload_t;
  void mq_payload_free(mq_payload_t *ptr);
  typedef mq_string_t mq_resource_descriptor_t;
  void mq_resource_descriptor_free(mq_resource_descriptor_t *ptr);
  mq_error_t mq_get_mq(mq_string_t *name, mq_resource_descriptor_t *ret0);
  mq_error_t mq_send(mq_resource_descriptor_t *rd, mq_payload_t *msg);
  mq_error_t mq_receive(mq_resource_descriptor_t *rd, mq_payload_t *ret0);
  #ifdef __cplusplus
}
#endif
#endif
