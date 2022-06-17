#include <stdlib.h>
#include <kv.h>

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

__attribute__((import_module("canonical_abi"), import_name("resource_drop_resource-descriptor")))
void __resource_resource_descriptor_drop(uint32_t idx);

void kv_resource_descriptor_free(kv_resource_descriptor_t *ptr) {
  __resource_resource_descriptor_drop(ptr->idx);
}

__attribute__((import_module("canonical_abi"), import_name("resource_clone_resource-descriptor")))
uint32_t __resource_resource_descriptor_clone(uint32_t idx);

kv_resource_descriptor_t kv_resource_descriptor_clone(kv_resource_descriptor_t *ptr) {
  return (kv_resource_descriptor_t){__resource_resource_descriptor_clone(ptr->idx)};
}
#include <string.h>

void kv_string_set(kv_string_t *ret, const char *s) {
  ret->ptr = (char*) s;
  ret->len = strlen(s);
}

void kv_string_dup(kv_string_t *ret, const char *s) {
  ret->len = strlen(s);
  ret->ptr = canonical_abi_realloc(NULL, 0, 1, ret->len);
  memcpy(ret->ptr, s, ret->len);
}

void kv_string_free(kv_string_t *ret) {
  canonical_abi_free(ret->ptr, ret->len, 1);
  ret->ptr = NULL;
  ret->len = 0;
}
void kv_payload_free(kv_payload_t *ptr) {
  canonical_abi_free(ptr->ptr, ptr->len * 1, 1);
}
typedef struct {
  bool is_err;
  union {
    kv_resource_descriptor_t ok;
    kv_error_t err;
  } val;
} kv_expected_resource_descriptor_error_t;
typedef struct {
  bool is_err;
  union {
    kv_payload_t ok;
    kv_error_t err;
  } val;
} kv_expected_payload_error_t;
typedef struct {
  bool is_err;
  union {
    kv_error_t err;
  } val;
} kv_expected_unit_error_t;

__attribute__((aligned(4)))
static uint8_t RET_AREA[12];
__attribute__((import_module("kv"), import_name("get-kv")))
void __wasm_import_kv_get_kv(int32_t);
kv_error_t kv_get_kv(kv_resource_descriptor_t *ret0) {
  int32_t ptr = (int32_t) &RET_AREA;
  __wasm_import_kv_get_kv(ptr);
  kv_expected_resource_descriptor_error_t expected;
  switch ((int32_t) (*((uint8_t*) (ptr + 0)))) {
    case 0: {
      expected.is_err = false;

      expected.val.ok = (kv_resource_descriptor_t){ *((int32_t*) (ptr + 4)) };
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
__attribute__((import_module("kv"), import_name("get")))
void __wasm_import_kv_get(int32_t, int32_t, int32_t, int32_t);
kv_error_t kv_get(kv_resource_descriptor_t rd, kv_string_t *key, kv_payload_t *ret0) {
  int32_t ptr = (int32_t) &RET_AREA;
  __wasm_import_kv_get((rd).idx, (int32_t) (*key).ptr, (int32_t) (*key).len, ptr);
  kv_expected_payload_error_t expected;
  switch ((int32_t) (*((uint8_t*) (ptr + 0)))) {
    case 0: {
      expected.is_err = false;

      expected.val.ok = (kv_payload_t) { (uint8_t*)(*((int32_t*) (ptr + 4))), (size_t)(*((int32_t*) (ptr + 8))) };
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
__attribute__((import_module("kv"), import_name("set")))
void __wasm_import_kv_set(int32_t, int32_t, int32_t, int32_t, int32_t, int32_t);
kv_error_t kv_set(kv_resource_descriptor_t rd, kv_string_t *key, kv_payload_t *value) {
  int32_t ptr = (int32_t) &RET_AREA;
  __wasm_import_kv_set((rd).idx, (int32_t) (*key).ptr, (int32_t) (*key).len, (int32_t) (*value).ptr, (int32_t) (*value).len, ptr);
  kv_expected_unit_error_t expected;
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
__attribute__((import_module("kv"), import_name("delete")))
void __wasm_import_kv_delete(int32_t, int32_t, int32_t, int32_t);
kv_error_t kv_delete(kv_resource_descriptor_t rd, kv_string_t *key) {
  int32_t ptr = (int32_t) &RET_AREA;
  __wasm_import_kv_delete((rd).idx, (int32_t) (*key).ptr, (int32_t) (*key).len, ptr);
  kv_expected_unit_error_t expected;
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
