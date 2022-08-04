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

__attribute__((import_module("canonical_abi"), import_name("resource_drop_mq")))
void __resource_mq_drop(uint32_t idx);

void mq_mq_free(mq_mq_t *ptr) {
  __resource_mq_drop(ptr->idx);
}

__attribute__((import_module("canonical_abi"), import_name("resource_clone_mq")))
uint32_t __resource_mq_clone(uint32_t idx);

mq_mq_t mq_mq_clone(mq_mq_t *ptr) {
  return (mq_mq_t){__resource_mq_clone(ptr->idx)};
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
void mq_error_free(mq_error_t *ptr) {
  switch ((int32_t) ptr->tag) {
    case 0: {
      mq_string_free(&ptr->val.error_with_description);
      break;
    }
  }
}
void mq_payload_free(mq_payload_t *ptr) {
  canonical_abi_free(ptr->ptr, ptr->len * 1, 1);
}
void mq_observable_free(mq_observable_t *ptr) {
  mq_string_free(&ptr->rd);
  mq_string_free(&ptr->key);
}
void mq_expected_mq_error_free(mq_expected_mq_error_t *ptr) {
  if (!ptr->is_err) {
    mq_mq_free(&ptr->val.ok);
  } else {
    mq_error_free(&ptr->val.err);
  }
}
void mq_expected_unit_error_free(mq_expected_unit_error_t *ptr) {
  if (!ptr->is_err) {
  } else {
    mq_error_free(&ptr->val.err);
  }
}
void mq_expected_payload_error_free(mq_expected_payload_error_t *ptr) {
  if (!ptr->is_err) {
    mq_payload_free(&ptr->val.ok);
  } else {
    mq_error_free(&ptr->val.err);
  }
}

__attribute__((aligned(4)))
static uint8_t RET_AREA[16];
__attribute__((import_module("mq"), import_name("mq::open")))
void __wasm_import_mq_mq_open(int32_t, int32_t, int32_t);
void mq_mq_open(mq_string_t *name, mq_expected_mq_error_t *ret0) {
  int32_t ptr = (int32_t) &RET_AREA;
  __wasm_import_mq_mq_open((int32_t) (*name).ptr, (int32_t) (*name).len, ptr);
  mq_expected_mq_error_t expected;
  switch ((int32_t) (*((uint8_t*) (ptr + 0)))) {
    case 0: {
      expected.is_err = false;
      
      expected.val.ok = (mq_mq_t){ *((int32_t*) (ptr + 4)) };
      break;
    }
    case 1: {
      expected.is_err = true;
      mq_error_t variant;
      variant.tag = (int32_t) (*((uint8_t*) (ptr + 4)));
      switch ((int32_t) variant.tag) {
        case 0: {
          variant.val.error_with_description = (mq_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
      }
      
      expected.val.err = variant;
      break;
    }
  }*ret0 = expected;
}
__attribute__((import_module("mq"), import_name("mq::send")))
void __wasm_import_mq_mq_send(int32_t, int32_t, int32_t, int32_t);
void mq_mq_send(mq_mq_t self, mq_payload_t *msg, mq_expected_unit_error_t *ret0) {
  int32_t ptr = (int32_t) &RET_AREA;
  __wasm_import_mq_mq_send((self).idx, (int32_t) (*msg).ptr, (int32_t) (*msg).len, ptr);
  mq_expected_unit_error_t expected;
  switch ((int32_t) (*((uint8_t*) (ptr + 0)))) {
    case 0: {
      expected.is_err = false;
      
      
      break;
    }
    case 1: {
      expected.is_err = true;
      mq_error_t variant;
      variant.tag = (int32_t) (*((uint8_t*) (ptr + 4)));
      switch ((int32_t) variant.tag) {
        case 0: {
          variant.val.error_with_description = (mq_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
      }
      
      expected.val.err = variant;
      break;
    }
  }*ret0 = expected;
}
__attribute__((import_module("mq"), import_name("mq::receive")))
void __wasm_import_mq_mq_receive(int32_t, int32_t);
void mq_mq_receive(mq_mq_t self, mq_expected_payload_error_t *ret0) {
  int32_t ptr = (int32_t) &RET_AREA;
  __wasm_import_mq_mq_receive((self).idx, ptr);
  mq_expected_payload_error_t expected;
  switch ((int32_t) (*((uint8_t*) (ptr + 0)))) {
    case 0: {
      expected.is_err = false;
      
      expected.val.ok = (mq_payload_t) { (uint8_t*)(*((int32_t*) (ptr + 4))), (size_t)(*((int32_t*) (ptr + 8))) };
      break;
    }
    case 1: {
      expected.is_err = true;
      mq_error_t variant;
      variant.tag = (int32_t) (*((uint8_t*) (ptr + 4)));
      switch ((int32_t) variant.tag) {
        case 0: {
          variant.val.error_with_description = (mq_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
      }
      
      expected.val.err = variant;
      break;
    }
  }*ret0 = expected;
}
