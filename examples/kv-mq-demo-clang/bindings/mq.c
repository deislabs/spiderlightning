#include <stdlib.h>
#include <mq.h>

__attribute__((weak, export_name("canonical_abi_realloc")))
void *canonical_abi_realloc(
void *ptr,
size_t orig_size,
size_t org_align,
size_t new_size
) {
  void *ret = realloc(ptr, new_size);
  if (!ret)
  abort();
  return ret;
}

__attribute__((weak, export_name("canonical_abi_free")))
void canonical_abi_free(
void *ptr,
size_t size,
size_t align
) {
  free(ptr);
}
#include <string.h>

void mq_string_set(mq_string_t *ret, const char *s) {
  ret->ptr = (char*) s;
  ret->len = strlen(s);
}

void mq_string_dup(mq_string_t *ret, const char *s) {
  ret->len = strlen(s);
  ret->ptr = canonical_abi_realloc(NULL, 0, 1, ret->len);
  memcpy(ret->ptr, s, ret->len);
}

void mq_string_free(mq_string_t *ret) {
  canonical_abi_free(ret->ptr, ret->len, 1);
  ret->ptr = NULL;
  ret->len = 0;
}
void mq_payload_free(mq_payload_t *ptr) {
  canonical_abi_free(ptr->ptr, ptr->len * 1, 1);
}
void mq_resource_descriptor_free(mq_resource_descriptor_t *ptr) {
  mq_string_free(ptr);
}
typedef struct {
  bool is_err;
  union {
    mq_resource_descriptor_t ok;
    mq_error_t err;
  } val;
} mq_expected_resource_descriptor_error_t;
typedef struct {
  bool is_err;
  union {
    mq_error_t err;
  } val;
} mq_expected_unit_error_t;
typedef struct {
  bool is_err;
  union {
    mq_payload_t ok;
    mq_error_t err;
  } val;
} mq_expected_payload_error_t;

__attribute__((aligned(4)))
static uint8_t RET_AREA[12];
__attribute__((import_module("mq"), import_name("get-mq")))
void __wasm_import_mq_get_mq(int32_t, int32_t, int32_t);
mq_error_t mq_get_mq(mq_string_t *name, mq_resource_descriptor_t *ret0) {
  int32_t ptr = (int32_t) &RET_AREA;
  __wasm_import_mq_get_mq((int32_t) (*name).ptr, (int32_t) (*name).len, ptr);
  mq_expected_resource_descriptor_error_t expected;
  switch ((int32_t) (*((uint8_t*) (ptr + 0)))) {
    case 0: {
      expected.is_err = false;
      
      expected.val.ok = (mq_string_t) { (char*)(*((int32_t*) (ptr + 4))), (size_t)(*((int32_t*) (ptr + 8))) };
      break;
    }
    case 1: {
      expected.is_err = true;
      
      expected.val.err = (int32_t) (*((uint8_t*) (ptr + 4)));
      break;
    }
  }*ret0 = expected.val.ok;
  return expected.is_err ? expected.val.err : -1;
}
__attribute__((import_module("mq"), import_name("send")))
void __wasm_import_mq_send(int32_t, int32_t, int32_t, int32_t, int32_t);
mq_error_t mq_send(mq_resource_descriptor_t *rd, mq_payload_t *msg) {
  int32_t ptr = (int32_t) &RET_AREA;
  __wasm_import_mq_send((int32_t) (*rd).ptr, (int32_t) (*rd).len, (int32_t) (*msg).ptr, (int32_t) (*msg).len, ptr);
  mq_expected_unit_error_t expected;
  switch ((int32_t) (*((uint8_t*) (ptr + 0)))) {
    case 0: {
      expected.is_err = false;
      
      
      break;
    }
    case 1: {
      expected.is_err = true;
      
      expected.val.err = (int32_t) (*((uint8_t*) (ptr + 1)));
      break;
    }
  }return expected.is_err ? expected.val.err : -1;
}
__attribute__((import_module("mq"), import_name("receive")))
void __wasm_import_mq_receive(int32_t, int32_t, int32_t);
mq_error_t mq_receive(mq_resource_descriptor_t *rd, mq_payload_t *ret0) {
  int32_t ptr = (int32_t) &RET_AREA;
  __wasm_import_mq_receive((int32_t) (*rd).ptr, (int32_t) (*rd).len, ptr);
  mq_expected_payload_error_t expected;
  switch ((int32_t) (*((uint8_t*) (ptr + 0)))) {
    case 0: {
      expected.is_err = false;
      
      expected.val.ok = (mq_payload_t) { (uint8_t*)(*((int32_t*) (ptr + 4))), (size_t)(*((int32_t*) (ptr + 8))) };
      break;
    }
    case 1: {
      expected.is_err = true;
      
      expected.val.err = (int32_t) (*((uint8_t*) (ptr + 4)));
      break;
    }
  }*ret0 = expected.val.ok;
  return expected.is_err ? expected.val.err : -1;
}
