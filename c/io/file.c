#include "c/file/file.h"

bool ufxr_writer_open(struct ufxr_writer *restrict fp, const char *path,
                      struct ufxr_error *err) {
    return true;
}

bool ufxr_writer_commit(struct ufxr_writer *restrict fp,
                        struct ufxr_error *err) {
    return true;
}

void ufxr_writer_close(struct ufxr_writer *restrict fp) {
    return;
}

bool ufxr_writer_write(struct ufxr_writer *restrict fp, const void *data,
                       size_t size) {
    return true;
}
