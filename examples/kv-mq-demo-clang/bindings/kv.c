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

__attribute__((import_module("canonical_abi"), import_name("resource_drop_kv")))
void __resource_kv_drop(uint32_t idx);

void kv_kv_free(kv_kv_t *ptr) {
  __resource_kv_drop(ptr->idx);
}

__attribute__((import_module("canonical_abi"), import_name("resource_clone_kv")))
uint32_t __resource_kv_clone(uint32_t idx);

kv_kv_t kv_kv_clone(kv_kv_t *ptr) {
  return (kv_kv_t){__resource_kv_clone(ptr->idx)};
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
void kv_error_free(kv_error_t *ptr) {
  switch ((int32_t) ptr->tag) {
    case 0: {
      kv_string_free(&ptr->val.error_with_description);
      break;
    }
  }
}
void kv_payload_free(kv_payload_t *ptr) {
  canonical_abi_free(ptr->ptr, ptr->len * 1, 1);
}
void kv_observable_free(kv_observable_t *ptr) {
  kv_string_free(&ptr->rd);
  kv_string_free(&ptr->key);
}
void kv_expected_kv_error_free(kv_expected_kv_error_t *ptr) {
  if (!ptr->is_err) {
    kv_kv_free(&ptr->val.ok);
  } else {
    kv_error_free(&ptr->val.err);
  }
}
void kv_expected_payload_error_free(kv_expected_payload_error_t *ptr) {
  if (!ptr->is_err) {
    kv_payload_free(&ptr->val.ok);
  } else {
    kv_error_free(&ptr->val.err);
  }
}
void kv_expected_unit_error_free(kv_expected_unit_error_t *ptr) {
  if (!ptr->is_err) {
  } else {
    kv_error_free(&ptr->val.err);
  }
}
void kv_expected_observable_error_free(kv_expected_observable_error_t *ptr) {
  if (!ptr->is_err) {
    kv_observable_free(&ptr->val.ok);
  } else {
    kv_error_free(&ptr->val.err);
  }
}

__attribute__((aligned(4)))
static uint8_t RET_AREA[20];
__attribute__((import_module("kv"), import_name("kv::open")))
void __wasm_import_kv_kv_open(int32_t, int32_t, int32_t);
void kv_kv_open(kv_string_t *name, kv_expected_kv_error_t *ret0) {
  int32_t ptr = (int32_t) &RET_AREA;
  __wasm_import_kv_kv_open((int32_t) (*name).ptr, (int32_t) (*name).len, ptr);
  kv_expected_kv_error_t expected;
  switch ((int32_t) (*((uint8_t*) (ptr + 0)))) {
    case 0: {
      expected.is_err = false;
      
      expected.val.ok = (kv_kv_t){ *((int32_t*) (ptr + 4)) };
      break;
    }
    case 1: {
      expected.is_err = true;
      kv_error_t variant;
      variant.tag = (int32_t) (*((uint8_t*) (ptr + 4)));
      switch ((int32_t) variant.tag) {
        case 0: {
          variant.val.error_with_description = (kv_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
      }
      
      expected.val.err = variant;
      break;
    }
  }*ret0 = expected;
}
__attribute__((import_module("kv"), import_name("kv::get")))
void __wasm_import_kv_kv_get(int32_t, int32_t, int32_t, int32_t);
void kv_kv_get(kv_kv_t self, kv_string_t *key, kv_expected_payload_error_t *ret0) {
  int32_t ptr = (int32_t) &RET_AREA;
  __wasm_import_kv_kv_get((self).idx, (int32_t) (*key).ptr, (int32_t) (*key).len, ptr);
  kv_expected_payload_error_t expected;
  switch ((int32_t) (*((uint8_t*) (ptr + 0)))) {
    case 0: {
      expected.is_err = false;
      
      expected.val.ok = (kv_payload_t) { (uint8_t*)(*((int32_t*) (ptr + 4))), (size_t)(*((int32_t*) (ptr + 8))) };
      break;
    }
    case 1: {
      expected.is_err = true;
      kv_error_t variant;
      variant.tag = (int32_t) (*((uint8_t*) (ptr + 4)));
      switch ((int32_t) variant.tag) {
        case 0: {
          variant.val.error_with_description = (kv_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
      }
      
      expected.val.err = variant;
      break;
    }
  }*ret0 = expected;
}
__attribute__((import_module("kv"), import_name("kv::set")))
void __wasm_import_kv_kv_set(int32_t, int32_t, int32_t, int32_t, int32_t, int32_t);
void kv_kv_set(kv_kv_t self, kv_string_t *key, kv_payload_t *value, kv_expected_unit_error_t *ret0) {
  int32_t ptr = (int32_t) &RET_AREA;
  __wasm_import_kv_kv_set((self).idx, (int32_t) (*key).ptr, (int32_t) (*key).len, (int32_t) (*value).ptr, (int32_t) (*value).len, ptr);
  kv_expected_unit_error_t expected;
  switch ((int32_t) (*((uint8_t*) (ptr + 0)))) {
    case 0: {
      expected.is_err = false;
      
      
      break;
    }
    case 1: {
      expected.is_err = true;
      kv_error_t variant;
      variant.tag = (int32_t) (*((uint8_t*) (ptr + 4)));
      switch ((int32_t) variant.tag) {
        case 0: {
          variant.val.error_with_description = (kv_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
      }
      
      expected.val.err = variant;
      break;
    }
  }*ret0 = expected;
}
__attribute__((import_module("kv"), import_name("kv::delete")))
void __wasm_import_kv_kv_delete(int32_t, int32_t, int32_t, int32_t);
void kv_kv_delete(kv_kv_t self, kv_string_t *key, kv_expected_unit_error_t *ret0) {
  int32_t ptr = (int32_t) &RET_AREA;
  __wasm_import_kv_kv_delete((self).idx, (int32_t) (*key).ptr, (int32_t) (*key).len, ptr);
  kv_expected_unit_error_t expected;
  switch ((int32_t) (*((uint8_t*) (ptr + 0)))) {
    case 0: {
      expected.is_err = false;
      
      
      break;
    }
    case 1: {
      expected.is_err = true;
      kv_error_t variant;
      variant.tag = (int32_t) (*((uint8_t*) (ptr + 4)));
      switch ((int32_t) variant.tag) {
        case 0: {
          variant.val.error_with_description = (kv_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
      }
      
      expected.val.err = variant;
      break;
    }
  }*ret0 = expected;
}
__attribute__((import_module("kv"), import_name("kv::watch")))
void __wasm_import_kv_kv_watch(int32_t, int32_t, int32_t, int32_t);
void kv_kv_watch(kv_kv_t self, kv_string_t *key, kv_expected_observable_error_t *ret0) {
  int32_t ptr = (int32_t) &RET_AREA;
  __wasm_import_kv_kv_watch((self).idx, (int32_t) (*key).ptr, (int32_t) (*key).len, ptr);
  kv_expected_observable_error_t expected;
  switch ((int32_t) (*((uint8_t*) (ptr + 0)))) {
    case 0: {
      expected.is_err = false;
      
      expected.val.ok = (kv_observable_t) {
        (kv_string_t) { (char*)(*((int32_t*) (ptr + 4))), (size_t)(*((int32_t*) (ptr + 8))) },
        (kv_string_t) { (char*)(*((int32_t*) (ptr + 12))), (size_t)(*((int32_t*) (ptr + 16))) },
      };
      break;
    }
    case 1: {
      expected.is_err = true;
      kv_error_t variant;
      variant.tag = (int32_t) (*((uint8_t*) (ptr + 4)));
      switch ((int32_t) variant.tag) {
        case 0: {
          variant.val.error_with_description = (kv_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
      }
      
      expected.val.err = variant;
      break;
    }
  }*ret0 = expected;
}
