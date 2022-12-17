#include <stdlib.h>
#include <messaging.h>

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

__attribute__((import_module("canonical_abi"), import_name("resource_drop_pub")))
void __resource_pub_drop(uint32_t idx);

void messaging_pub_free(messaging_pub_t *ptr) {
  __resource_pub_drop(ptr->idx);
}

__attribute__((import_module("canonical_abi"), import_name("resource_clone_pub")))
uint32_t __resource_pub_clone(uint32_t idx);

messaging_pub_t messaging_pub_clone(messaging_pub_t *ptr) {
  return (messaging_pub_t){__resource_pub_clone(ptr->idx)};
}

__attribute__((import_module("canonical_abi"), import_name("resource_drop_sub")))
void __resource_sub_drop(uint32_t idx);

void messaging_sub_free(messaging_sub_t *ptr) {
  __resource_sub_drop(ptr->idx);
}

__attribute__((import_module("canonical_abi"), import_name("resource_clone_sub")))
uint32_t __resource_sub_clone(uint32_t idx);

messaging_sub_t messaging_sub_clone(messaging_sub_t *ptr) {
  return (messaging_sub_t){__resource_sub_clone(ptr->idx)};
}
#include <string.h>

void messaging_string_set(messaging_string_t *ret, const char *s) {
  ret->ptr = (char*) s;
  ret->len = strlen(s);
}

void messaging_string_dup(messaging_string_t *ret, const char *s) {
  ret->len = strlen(s);
  ret->ptr = canonical_abi_realloc(NULL, 0, 1, ret->len);
  memcpy(ret->ptr, s, ret->len);
}

void messaging_string_free(messaging_string_t *ret) {
  canonical_abi_free(ret->ptr, ret->len, 1);
  ret->ptr = NULL;
  ret->len = 0;
}
void messaging_messaging_error_free(messaging_messaging_error_t *ptr) {
  switch ((int32_t) ptr->tag) {
    case 0: {
      messaging_string_free(&ptr->val.payload_too_large);
      break;
    }
    case 1: {
      messaging_string_free(&ptr->val.queue_or_topic_not_found);
      break;
    }
    case 2: {
      messaging_string_free(&ptr->val.insufficient_permissions);
      break;
    }
    case 3: {
      messaging_string_free(&ptr->val.service_unavailable);
      break;
    }
    case 4: {
      messaging_string_free(&ptr->val.delivery_failed);
      break;
    }
    case 5: {
      messaging_string_free(&ptr->val.connection_lost);
      break;
    }
    case 6: {
      messaging_string_free(&ptr->val.unsupported_message_format);
      break;
    }
    case 7: {
      messaging_string_free(&ptr->val.unexpected_error);
      break;
    }
  }
}
void messaging_subscription_token_free(messaging_subscription_token_t *ptr) {
  messaging_string_free(ptr);
}
void messaging_expected_pub_messaging_error_free(messaging_expected_pub_messaging_error_t *ptr) {
  if (!ptr->is_err) {
    messaging_pub_free(&ptr->val.ok);
  } else {
    messaging_messaging_error_free(&ptr->val.err);
  }
}
void messaging_list_u8_free(messaging_list_u8_t *ptr) {
  canonical_abi_free(ptr->ptr, ptr->len * 1, 1);
}
void messaging_expected_unit_messaging_error_free(messaging_expected_unit_messaging_error_t *ptr) {
  if (!ptr->is_err) {
  } else {
    messaging_messaging_error_free(&ptr->val.err);
  }
}
void messaging_expected_sub_messaging_error_free(messaging_expected_sub_messaging_error_t *ptr) {
  if (!ptr->is_err) {
    messaging_sub_free(&ptr->val.ok);
  } else {
    messaging_messaging_error_free(&ptr->val.err);
  }
}
void messaging_expected_subscription_token_messaging_error_free(messaging_expected_subscription_token_messaging_error_t *ptr) {
  if (!ptr->is_err) {
    messaging_subscription_token_free(&ptr->val.ok);
  } else {
    messaging_messaging_error_free(&ptr->val.err);
  }
}
void messaging_expected_list_u8_messaging_error_free(messaging_expected_list_u8_messaging_error_t *ptr) {
  if (!ptr->is_err) {
    messaging_list_u8_free(&ptr->val.ok);
  } else {
    messaging_messaging_error_free(&ptr->val.err);
  }
}

__attribute__((aligned(4)))
static uint8_t RET_AREA[16];
__attribute__((import_module("messaging"), import_name("pub::open")))
void __wasm_import_messaging_pub_open(int32_t, int32_t, int32_t);
void messaging_pub_open(messaging_string_t *name, messaging_expected_pub_messaging_error_t *ret0) {
  int32_t ptr = (int32_t) &RET_AREA;
  __wasm_import_messaging_pub_open((int32_t) (*name).ptr, (int32_t) (*name).len, ptr);
  messaging_expected_pub_messaging_error_t expected;
  switch ((int32_t) (*((uint8_t*) (ptr + 0)))) {
    case 0: {
      expected.is_err = false;
      
      expected.val.ok = (messaging_pub_t){ *((int32_t*) (ptr + 4)) };
      break;
    }
    case 1: {
      expected.is_err = true;
      messaging_messaging_error_t variant;
      variant.tag = (int32_t) (*((uint8_t*) (ptr + 4)));
      switch ((int32_t) variant.tag) {
        case 0: {
          variant.val.payload_too_large = (messaging_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 1: {
          variant.val.queue_or_topic_not_found = (messaging_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 2: {
          variant.val.insufficient_permissions = (messaging_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 3: {
          variant.val.service_unavailable = (messaging_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 4: {
          variant.val.delivery_failed = (messaging_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 5: {
          variant.val.connection_lost = (messaging_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 6: {
          variant.val.unsupported_message_format = (messaging_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 7: {
          variant.val.unexpected_error = (messaging_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
      }
      
      expected.val.err = variant;
      break;
    }
  }*ret0 = expected;
}
__attribute__((import_module("messaging"), import_name("pub::publish")))
void __wasm_import_messaging_pub_publish(int32_t, int32_t, int32_t, int32_t, int32_t, int32_t);
void messaging_pub_publish(messaging_pub_t self, messaging_list_u8_t *msg, messaging_string_t *topic, messaging_expected_unit_messaging_error_t *ret0) {
  int32_t ptr = (int32_t) &RET_AREA;
  __wasm_import_messaging_pub_publish((self).idx, (int32_t) (*msg).ptr, (int32_t) (*msg).len, (int32_t) (*topic).ptr, (int32_t) (*topic).len, ptr);
  messaging_expected_unit_messaging_error_t expected;
  switch ((int32_t) (*((uint8_t*) (ptr + 0)))) {
    case 0: {
      expected.is_err = false;
      
      
      break;
    }
    case 1: {
      expected.is_err = true;
      messaging_messaging_error_t variant;
      variant.tag = (int32_t) (*((uint8_t*) (ptr + 4)));
      switch ((int32_t) variant.tag) {
        case 0: {
          variant.val.payload_too_large = (messaging_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 1: {
          variant.val.queue_or_topic_not_found = (messaging_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 2: {
          variant.val.insufficient_permissions = (messaging_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 3: {
          variant.val.service_unavailable = (messaging_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 4: {
          variant.val.delivery_failed = (messaging_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 5: {
          variant.val.connection_lost = (messaging_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 6: {
          variant.val.unsupported_message_format = (messaging_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 7: {
          variant.val.unexpected_error = (messaging_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
      }
      
      expected.val.err = variant;
      break;
    }
  }*ret0 = expected;
}
__attribute__((import_module("messaging"), import_name("sub::open")))
void __wasm_import_messaging_sub_open(int32_t, int32_t, int32_t);
void messaging_sub_open(messaging_string_t *name, messaging_expected_sub_messaging_error_t *ret0) {
  int32_t ptr = (int32_t) &RET_AREA;
  __wasm_import_messaging_sub_open((int32_t) (*name).ptr, (int32_t) (*name).len, ptr);
  messaging_expected_sub_messaging_error_t expected;
  switch ((int32_t) (*((uint8_t*) (ptr + 0)))) {
    case 0: {
      expected.is_err = false;
      
      expected.val.ok = (messaging_sub_t){ *((int32_t*) (ptr + 4)) };
      break;
    }
    case 1: {
      expected.is_err = true;
      messaging_messaging_error_t variant;
      variant.tag = (int32_t) (*((uint8_t*) (ptr + 4)));
      switch ((int32_t) variant.tag) {
        case 0: {
          variant.val.payload_too_large = (messaging_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 1: {
          variant.val.queue_or_topic_not_found = (messaging_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 2: {
          variant.val.insufficient_permissions = (messaging_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 3: {
          variant.val.service_unavailable = (messaging_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 4: {
          variant.val.delivery_failed = (messaging_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 5: {
          variant.val.connection_lost = (messaging_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 6: {
          variant.val.unsupported_message_format = (messaging_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 7: {
          variant.val.unexpected_error = (messaging_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
      }
      
      expected.val.err = variant;
      break;
    }
  }*ret0 = expected;
}
__attribute__((import_module("messaging"), import_name("sub::subscribe")))
void __wasm_import_messaging_sub_subscribe(int32_t, int32_t, int32_t, int32_t);
void messaging_sub_subscribe(messaging_sub_t self, messaging_string_t *topic, messaging_expected_subscription_token_messaging_error_t *ret0) {
  int32_t ptr = (int32_t) &RET_AREA;
  __wasm_import_messaging_sub_subscribe((self).idx, (int32_t) (*topic).ptr, (int32_t) (*topic).len, ptr);
  messaging_expected_subscription_token_messaging_error_t expected;
  switch ((int32_t) (*((uint8_t*) (ptr + 0)))) {
    case 0: {
      expected.is_err = false;
      
      expected.val.ok = (messaging_string_t) { (char*)(*((int32_t*) (ptr + 4))), (size_t)(*((int32_t*) (ptr + 8))) };
      break;
    }
    case 1: {
      expected.is_err = true;
      messaging_messaging_error_t variant;
      variant.tag = (int32_t) (*((uint8_t*) (ptr + 4)));
      switch ((int32_t) variant.tag) {
        case 0: {
          variant.val.payload_too_large = (messaging_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 1: {
          variant.val.queue_or_topic_not_found = (messaging_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 2: {
          variant.val.insufficient_permissions = (messaging_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 3: {
          variant.val.service_unavailable = (messaging_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 4: {
          variant.val.delivery_failed = (messaging_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 5: {
          variant.val.connection_lost = (messaging_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 6: {
          variant.val.unsupported_message_format = (messaging_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 7: {
          variant.val.unexpected_error = (messaging_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
      }
      
      expected.val.err = variant;
      break;
    }
  }*ret0 = expected;
}
__attribute__((import_module("messaging"), import_name("sub::receive")))
void __wasm_import_messaging_sub_receive(int32_t, int32_t, int32_t, int32_t);
void messaging_sub_receive(messaging_sub_t self, messaging_subscription_token_t *sub_tok, messaging_expected_list_u8_messaging_error_t *ret0) {
  int32_t ptr = (int32_t) &RET_AREA;
  __wasm_import_messaging_sub_receive((self).idx, (int32_t) (*sub_tok).ptr, (int32_t) (*sub_tok).len, ptr);
  messaging_expected_list_u8_messaging_error_t expected;
  switch ((int32_t) (*((uint8_t*) (ptr + 0)))) {
    case 0: {
      expected.is_err = false;
      
      expected.val.ok = (messaging_list_u8_t) { (uint8_t*)(*((int32_t*) (ptr + 4))), (size_t)(*((int32_t*) (ptr + 8))) };
      break;
    }
    case 1: {
      expected.is_err = true;
      messaging_messaging_error_t variant;
      variant.tag = (int32_t) (*((uint8_t*) (ptr + 4)));
      switch ((int32_t) variant.tag) {
        case 0: {
          variant.val.payload_too_large = (messaging_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 1: {
          variant.val.queue_or_topic_not_found = (messaging_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 2: {
          variant.val.insufficient_permissions = (messaging_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 3: {
          variant.val.service_unavailable = (messaging_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 4: {
          variant.val.delivery_failed = (messaging_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 5: {
          variant.val.connection_lost = (messaging_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 6: {
          variant.val.unsupported_message_format = (messaging_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
        case 7: {
          variant.val.unexpected_error = (messaging_string_t) { (char*)(*((int32_t*) (ptr + 8))), (size_t)(*((int32_t*) (ptr + 12))) };
          break;
        }
      }
      
      expected.val.err = variant;
      break;
    }
  }*ret0 = expected;
}
