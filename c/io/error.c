// error.c - Error types and error handling.
#include "c/io/error.h"

#include <errno.h>

// Instantiate inline functions

void ufxr_error_setcode(struct ufxr_error *err, ufxr_errcode code);
void ufxr_error_setsystem(struct ufxr_error *err, int code);

void ufxr_error_seterrno(struct ufxr_error *err) {
    ufxr_error_setsystem(err, errno);
}
