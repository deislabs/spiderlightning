#include <stdlib.h>
#include <state-store.h>

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

__attribute__((import_module("canonical_abi"), import_name("resource_drop_state-store")))
void __resource_state_store_drop(uint32_t idx);

void state_store_state_store_free(state_store_state_store_t *ptr) {
  __resource_state_store_drop(ptr->idx);
}

__attribute__((import_module("canonical_abi"), import_name("resource_clone_state-store")))
uint32_t __resource_state_store_clone(uint32_t idx);

state_store_state_store_t state_store_state_store_clone(state_store_state_store_t *ptr) {
  return (state_store_state_store_t){__resource_state_store_clone(ptr->idx)};
}
#include <string.h>

void state_store_string_set(state_store_string_t *ret, const char *s) {
  ret->ptr = (char*) s;
  ret->len = strlen(s);
}

void state_store_string_dup(state_store_string_t *ret, const char *s) {
  ret->len = strlen(s);
  ret->ptr = canonical_abi_realloc(NULL, 0, 1, ret->len);
  memcpy(ret->ptr, s, ret->len);
}

void state_store_string_free(state_store_string_t *ret) {
  canonical_abi_free(ret->ptr, ret->len, 1);
  ret->ptr = NULL;
  ret->len = 0;
}
void state_store_error_free(state_store_error_t *ptr) {
  switch ((int32_t) ptr->tag) {
    case 0: {
      state_store_string_free(&ptr->val.error_with_description);
      break;
    }
  }
}
void state_store_payload_free(state_store_payload_t *ptr) {
  canonical_abi_free(ptr->ptr, ptr->len * 1, 1);
}
void state_store_observable_free(state_store_observable_t *ptr) {
  state_store_string_free(&ptr->rd);
  state_store_string_free(&ptr->key);
}
void state_store_expected_state_store_error_free(state_store_expected_state_store_error_t *ptr) {
  if (!ptr->is_err) {
    state_store_state_store_free(&ptr->val.ok);
  } else {
    state_store_error_free(&ptr->val.err);
  }
}
void state_store_expected_payload_error_free(state_store_expected_payload_error_t *ptr) {
  if (!ptr->is_err) {
    state_store_payload_free(&ptr->val.ok);
  } else {
    state_store_error_free(&ptr->val.err);
  }
}
void state_store_expected_unit_error_free(state_store_expected_unit_error_t *ptr) {
  if (!ptr->is_err) {
  } else {
    state_store_error_free(&ptr->val.err);
  }
}
void state_store_expected_observable_error_free(state_store_expected_observable_error_t *ptr) {
  if (!ptr->is_err) {
    state_store_observable_free(&ptr->val.ok);
  } else {
    state_store_error_free(&ptr->val.err);
  }
}

__attribute__((aligned(4)))
static uint8_t RET_AREA[20];
__attribute__((import_module("state_store"), import_name("state-store::open")))
void __wasm_import_state_store_state_store_open(int32_t, int32_t, int32_t);
void state_store_state_store_open(state_store_string_t *name, state_store_expected_state_store_error_t *ret0) {
  int32_t ptr = (int32_t) &RET_AREA;
  __wasm_import_state_store_state_store_open((int32_t) (*name).ptr, (int32_t) (*name).len, ptr);
  state_store_expected_state_store_error_t expected;
  switch ((int32_t) (*((uint8_t*) (ptr + 0)))) {
    case 0: {
      expected.is_err = false;
      
      expected.val.ok = (state_store_state_store_t){ *((int32_t*) (ptr + 4)) };
      break;
    }
    case 1: {
      expected.is_err = true;
      state_store_error_t variant;
      variant.tag = (int32_t) (*((uint8_t*) (ptr + 4)));
      switch ((int32_t) variant.tag) {
        case 0: {
          variant.val.error_with_description = (state_store_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
      }
      
      expected.val.err = variant;
      break;
    }
  }*ret0 = expected;
}
__attribute__((import_module("state_store"), import_name("state-store::get")))
void __wasm_import_state_store_state_store_get(int32_t, int32_t, int32_t, int32_t);
void state_store_state_store_get(state_store_state_store_t self, state_store_string_t *key, state_store_expected_payload_error_t *ret0) {
  int32_t ptr = (int32_t) &RET_AREA;
  __wasm_import_state_store_state_store_get((self).idx, (int32_t) (*key).ptr, (int32_t) (*key).len, ptr);
  state_store_expected_payload_error_t expected;
  switch ((int32_t) (*((uint8_t*) (ptr + 0)))) {
    case 0: {
      expected.is_err = false;
      
      expected.val.ok = (state_store_payload_t) { (uint8_t*)(*((int32_t*) (ptr + 4))), (size_t)(*((int32_t*) (ptr + 8))) };
      break;
    }
    case 1: {
      expected.is_err = true;
      state_store_error_t variant;
      variant.tag = (int32_t) (*((uint8_t*) (ptr + 4)));
      switch ((int32_t) variant.tag) {
        case 0: {
          variant.val.error_with_description = (state_store_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
      }
      
      expected.val.err = variant;
      break;
    }
  }*ret0 = expected;
}
__attribute__((import_module("state_store"), import_name("state-store::set")))
void __wasm_import_state_store_state_store_set(int32_t, int32_t, int32_t, int32_t, int32_t, int32_t);
void state_store_state_store_set(state_store_state_store_t self, state_store_string_t *key, state_store_payload_t *value, state_store_expected_unit_error_t *ret0) {
  int32_t ptr = (int32_t) &RET_AREA;
  __wasm_import_state_store_state_store_set((self).idx, (int32_t) (*key).ptr, (int32_t) (*key).len, (int32_t) (*value).ptr, (int32_t) (*value).len, ptr);
  state_store_expected_unit_error_t expected;
  switch ((int32_t) (*((uint8_t*) (ptr + 0)))) {
    case 0: {
      expected.is_err = false;
      
      
      break;
    }
    case 1: {
      expected.is_err = true;
      state_store_error_t variant;
      variant.tag = (int32_t) (*((uint8_t*) (ptr + 4)));
      switch ((int32_t) variant.tag) {
        case 0: {
          variant.val.error_with_description = (state_store_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
      }
      
      expected.val.err = variant;
      break;
    }
  }*ret0 = expected;
}
__attribute__((import_module("state_store"), import_name("state-store::delete")))
void __wasm_import_state_store_state_store_delete(int32_t, int32_t, int32_t, int32_t);
void state_store_state_store_delete(state_store_state_store_t self, state_store_string_t *key, state_store_expected_unit_error_t *ret0) {
  int32_t ptr = (int32_t) &RET_AREA;
  __wasm_import_state_store_state_store_delete((self).idx, (int32_t) (*key).ptr, (int32_t) (*key).len, ptr);
  state_store_expected_unit_error_t expected;
  switch ((int32_t) (*((uint8_t*) (ptr + 0)))) {
    case 0: {
      expected.is_err = false;
      
      
      break;
    }
    case 1: {
      expected.is_err = true;
      state_store_error_t variant;
      variant.tag = (int32_t) (*((uint8_t*) (ptr + 4)));
      switch ((int32_t) variant.tag) {
        case 0: {
          variant.val.error_with_description = (state_store_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
      }
      
      expected.val.err = variant;
      break;
    }
  }*ret0 = expected;
}
__attribute__((import_module("state_store"), import_name("state-store::watch")))
void __wasm_import_state_store_state_store_watch(int32_t, int32_t, int32_t, int32_t);
void state_store_state_store_watch(state_store_state_store_t self, state_store_string_t *key, state_store_expected_observable_error_t *ret0) {
  int32_t ptr = (int32_t) &RET_AREA;
  __wasm_import_state_store_state_store_watch((self).idx, (int32_t) (*key).ptr, (int32_t) (*key).len, ptr);
  state_store_expected_observable_error_t expected;
  switch ((int32_t) (*((uint8_t*) (ptr + 0)))) {
    case 0: {
      expected.is_err = false;
      
      expected.val.ok = (state_store_observable_t) {
        (state_store_string_t) { (char*)(*((int32_t*) (ptr + 4))), (size_t)(*((int32_t*) (ptr + 8))) },
        (state_store_string_t) { (char*)(*((int32_t*) (ptr + 12))), (size_t)(*((int32_t*) (ptr + 16))) },
      };
      break;
    }
    case 1: {
      expected.is_err = true;
      state_store_error_t variant;
      variant.tag = (int32_t) (*((uint8_t*) (ptr + 4)));
      switch ((int32_t) variant.tag) {
        case 0: {
          variant.val.error_with_description = (state_store_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
      }
      
      expected.val.err = variant;
      break;
    }
  }*ret0 = expected;
}
