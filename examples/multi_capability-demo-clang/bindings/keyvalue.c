#include <stdlib.h>
#include <keyvalue.h>

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

__attribute__((import_module("canonical_abi"), import_name("resource_drop_keyvalue")))
void __resource_keyvalue_drop(uint32_t idx);

void keyvalue_keyvalue_free(keyvalue_keyvalue_t *ptr) {
  __resource_keyvalue_drop(ptr->idx);
}

__attribute__((import_module("canonical_abi"), import_name("resource_clone_keyvalue")))
uint32_t __resource_keyvalue_clone(uint32_t idx);

keyvalue_keyvalue_t keyvalue_keyvalue_clone(keyvalue_keyvalue_t *ptr) {
  return (keyvalue_keyvalue_t){__resource_keyvalue_clone(ptr->idx)};
}
#include <string.h>

void keyvalue_string_set(keyvalue_string_t *ret, const char *s) {
  ret->ptr = (char*) s;
  ret->len = strlen(s);
}

void keyvalue_string_dup(keyvalue_string_t *ret, const char *s) {
  ret->len = strlen(s);
  ret->ptr = canonical_abi_realloc(NULL, 0, 1, ret->len);
  memcpy(ret->ptr, s, ret->len);
}

void keyvalue_string_free(keyvalue_string_t *ret) {
  canonical_abi_free(ret->ptr, ret->len, 1);
  ret->ptr = NULL;
  ret->len = 0;
}
void keyvalue_keyvalue_error_free(keyvalue_keyvalue_error_t *ptr) {
  switch ((int32_t) ptr->tag) {
    case 0: {
      keyvalue_string_free(&ptr->val.key_not_found);
      break;
    }
    case 1: {
      keyvalue_string_free(&ptr->val.invalid_key);
      break;
    }
    case 2: {
      keyvalue_string_free(&ptr->val.invalid_value);
      break;
    }
    case 3: {
      keyvalue_string_free(&ptr->val.connection_error);
      break;
    }
    case 4: {
      keyvalue_string_free(&ptr->val.authentication_error);
      break;
    }
    case 5: {
      keyvalue_string_free(&ptr->val.timeout_error);
      break;
    }
    case 6: {
      keyvalue_string_free(&ptr->val.io_error);
      break;
    }
    case 7: {
      keyvalue_string_free(&ptr->val.unexpected_error);
      break;
    }
  }
}
void keyvalue_expected_keyvalue_keyvalue_error_free(keyvalue_expected_keyvalue_keyvalue_error_t *ptr) {
  if (!ptr->is_err) {
    keyvalue_keyvalue_free(&ptr->val.ok);
  } else {
    keyvalue_keyvalue_error_free(&ptr->val.err);
  }
}
void keyvalue_list_u8_free(keyvalue_list_u8_t *ptr) {
  canonical_abi_free(ptr->ptr, ptr->len * 1, 1);
}
void keyvalue_expected_list_u8_keyvalue_error_free(keyvalue_expected_list_u8_keyvalue_error_t *ptr) {
  if (!ptr->is_err) {
    keyvalue_list_u8_free(&ptr->val.ok);
  } else {
    keyvalue_keyvalue_error_free(&ptr->val.err);
  }
}
void keyvalue_expected_unit_keyvalue_error_free(keyvalue_expected_unit_keyvalue_error_t *ptr) {
  if (!ptr->is_err) {
  } else {
    keyvalue_keyvalue_error_free(&ptr->val.err);
  }
}
void keyvalue_list_string_free(keyvalue_list_string_t *ptr) {
  for (size_t i = 0; i < ptr->len; i++) {
    keyvalue_string_free(&ptr->ptr[i]);
  }
  canonical_abi_free(ptr->ptr, ptr->len * 8, 4);
}
void keyvalue_expected_list_string_keyvalue_error_free(keyvalue_expected_list_string_keyvalue_error_t *ptr) {
  if (!ptr->is_err) {
    keyvalue_list_string_free(&ptr->val.ok);
  } else {
    keyvalue_keyvalue_error_free(&ptr->val.err);
  }
}

__attribute__((aligned(4)))
static uint8_t RET_AREA[16];
__attribute__((import_module("keyvalue"), import_name("keyvalue::open")))
void __wasm_import_keyvalue_keyvalue_open(int32_t, int32_t, int32_t);
void keyvalue_keyvalue_open(keyvalue_string_t *name, keyvalue_expected_keyvalue_keyvalue_error_t *ret0) {
  int32_t ptr = (int32_t) &RET_AREA;
  __wasm_import_keyvalue_keyvalue_open((int32_t) (*name).ptr, (int32_t) (*name).len, ptr);
  keyvalue_expected_keyvalue_keyvalue_error_t expected;
  switch ((int32_t) (*((uint8_t*) (ptr + 0)))) {
    case 0: {
      expected.is_err = false;
      
      expected.val.ok = (keyvalue_keyvalue_t){ *((int32_t*) (ptr + 4)) };
      break;
    }
    case 1: {
      expected.is_err = true;
      keyvalue_keyvalue_error_t variant;
      variant.tag = (int32_t) (*((uint8_t*) (ptr + 4)));
      switch ((int32_t) variant.tag) {
        case 0: {
          variant.val.key_not_found = (keyvalue_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 1: {
          variant.val.invalid_key = (keyvalue_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 2: {
          variant.val.invalid_value = (keyvalue_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 3: {
          variant.val.connection_error = (keyvalue_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 4: {
          variant.val.authentication_error = (keyvalue_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 5: {
          variant.val.timeout_error = (keyvalue_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 6: {
          variant.val.io_error = (keyvalue_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 7: {
          variant.val.unexpected_error = (keyvalue_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
      }
      
      expected.val.err = variant;
      break;
    }
  }*ret0 = expected;
}
__attribute__((import_module("keyvalue"), import_name("keyvalue::get")))
void __wasm_import_keyvalue_keyvalue_get(int32_t, int32_t, int32_t, int32_t);
void keyvalue_keyvalue_get(keyvalue_keyvalue_t self, keyvalue_string_t *key, keyvalue_expected_list_u8_keyvalue_error_t *ret0) {
  int32_t ptr = (int32_t) &RET_AREA;
  __wasm_import_keyvalue_keyvalue_get((self).idx, (int32_t) (*key).ptr, (int32_t) (*key).len, ptr);
  keyvalue_expected_list_u8_keyvalue_error_t expected;
  switch ((int32_t) (*((uint8_t*) (ptr + 0)))) {
    case 0: {
      expected.is_err = false;
      
      expected.val.ok = (keyvalue_list_u8_t) { (uint8_t*)(*((int32_t*) (ptr + 4))), (size_t)(*((int32_t*) (ptr + 8))) };
      break;
    }
    case 1: {
      expected.is_err = true;
      keyvalue_keyvalue_error_t variant;
      variant.tag = (int32_t) (*((uint8_t*) (ptr + 4)));
      switch ((int32_t) variant.tag) {
        case 0: {
          variant.val.key_not_found = (keyvalue_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 1: {
          variant.val.invalid_key = (keyvalue_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 2: {
          variant.val.invalid_value = (keyvalue_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 3: {
          variant.val.connection_error = (keyvalue_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 4: {
          variant.val.authentication_error = (keyvalue_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 5: {
          variant.val.timeout_error = (keyvalue_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 6: {
          variant.val.io_error = (keyvalue_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 7: {
          variant.val.unexpected_error = (keyvalue_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
      }
      
      expected.val.err = variant;
      break;
    }
  }*ret0 = expected;
}
__attribute__((import_module("keyvalue"), import_name("keyvalue::set")))
void __wasm_import_keyvalue_keyvalue_set(int32_t, int32_t, int32_t, int32_t, int32_t, int32_t);
void keyvalue_keyvalue_set(keyvalue_keyvalue_t self, keyvalue_string_t *key, keyvalue_list_u8_t *value, keyvalue_expected_unit_keyvalue_error_t *ret0) {
  int32_t ptr = (int32_t) &RET_AREA;
  __wasm_import_keyvalue_keyvalue_set((self).idx, (int32_t) (*key).ptr, (int32_t) (*key).len, (int32_t) (*value).ptr, (int32_t) (*value).len, ptr);
  keyvalue_expected_unit_keyvalue_error_t expected;
  switch ((int32_t) (*((uint8_t*) (ptr + 0)))) {
    case 0: {
      expected.is_err = false;
      
      
      break;
    }
    case 1: {
      expected.is_err = true;
      keyvalue_keyvalue_error_t variant;
      variant.tag = (int32_t) (*((uint8_t*) (ptr + 4)));
      switch ((int32_t) variant.tag) {
        case 0: {
          variant.val.key_not_found = (keyvalue_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 1: {
          variant.val.invalid_key = (keyvalue_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 2: {
          variant.val.invalid_value = (keyvalue_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 3: {
          variant.val.connection_error = (keyvalue_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 4: {
          variant.val.authentication_error = (keyvalue_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 5: {
          variant.val.timeout_error = (keyvalue_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 6: {
          variant.val.io_error = (keyvalue_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 7: {
          variant.val.unexpected_error = (keyvalue_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
      }
      
      expected.val.err = variant;
      break;
    }
  }*ret0 = expected;
}
__attribute__((import_module("keyvalue"), import_name("keyvalue::keys")))
void __wasm_import_keyvalue_keyvalue_keys(int32_t, int32_t);
void keyvalue_keyvalue_keys(keyvalue_keyvalue_t self, keyvalue_expected_list_string_keyvalue_error_t *ret0) {
  int32_t ptr = (int32_t) &RET_AREA;
  __wasm_import_keyvalue_keyvalue_keys((self).idx, ptr);
  keyvalue_expected_list_string_keyvalue_error_t expected;
  switch ((int32_t) (*((uint8_t*) (ptr + 0)))) {
    case 0: {
      expected.is_err = false;
      
      expected.val.ok = (keyvalue_list_string_t) { (keyvalue_string_t*)(*((int32_t*) (ptr + 4))), (size_t)(*((int32_t*) (ptr + 8))) };
      break;
    }
    case 1: {
      expected.is_err = true;
      keyvalue_keyvalue_error_t variant;
      variant.tag = (int32_t) (*((uint8_t*) (ptr + 4)));
      switch ((int32_t) variant.tag) {
        case 0: {
          variant.val.key_not_found = (keyvalue_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 1: {
          variant.val.invalid_key = (keyvalue_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 2: {
          variant.val.invalid_value = (keyvalue_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 3: {
          variant.val.connection_error = (keyvalue_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 4: {
          variant.val.authentication_error = (keyvalue_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 5: {
          variant.val.timeout_error = (keyvalue_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 6: {
          variant.val.io_error = (keyvalue_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 7: {
          variant.val.unexpected_error = (keyvalue_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
      }
      
      expected.val.err = variant;
      break;
    }
  }*ret0 = expected;
}
__attribute__((import_module("keyvalue"), import_name("keyvalue::delete")))
void __wasm_import_keyvalue_keyvalue_delete(int32_t, int32_t, int32_t, int32_t);
void keyvalue_keyvalue_delete(keyvalue_keyvalue_t self, keyvalue_string_t *key, keyvalue_expected_unit_keyvalue_error_t *ret0) {
  int32_t ptr = (int32_t) &RET_AREA;
  __wasm_import_keyvalue_keyvalue_delete((self).idx, (int32_t) (*key).ptr, (int32_t) (*key).len, ptr);
  keyvalue_expected_unit_keyvalue_error_t expected;
  switch ((int32_t) (*((uint8_t*) (ptr + 0)))) {
    case 0: {
      expected.is_err = false;
      
      
      break;
    }
    case 1: {
      expected.is_err = true;
      keyvalue_keyvalue_error_t variant;
      variant.tag = (int32_t) (*((uint8_t*) (ptr + 4)));
      switch ((int32_t) variant.tag) {
        case 0: {
          variant.val.key_not_found = (keyvalue_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 1: {
          variant.val.invalid_key = (keyvalue_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 2: {
          variant.val.invalid_value = (keyvalue_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 3: {
          variant.val.connection_error = (keyvalue_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 4: {
          variant.val.authentication_error = (keyvalue_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 5: {
          variant.val.timeout_error = (keyvalue_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 6: {
          variant.val.io_error = (keyvalue_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 7: {
          variant.val.unexpected_error = (keyvalue_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
      }
      
      expected.val.err = variant;
      break;
    }
  }*ret0 = expected;
}
