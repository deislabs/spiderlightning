package main

// #include "config.h"
// #include "blob.h"
// #include<stdlib.h>
// #include<stddef.h>
import "C"
import "fmt"

func config_get_capability(config *C.config_map_t) C.config_error_t {
	var ptr = C.config_string_t{}
	var pt2 = C.config_string_t{}
	property := C.config_string_set()
	// value := C.config_string_set(&ptr, C.CString("."))

	// C.config_tuple2_string_string_t{
	// 	f0: ptr,
	// 	f1: pt2,
	// }
	return C.uint8_t(0)
}

func main() {
	// get_blob()
	fmt.Println("Hello, world.")
}
