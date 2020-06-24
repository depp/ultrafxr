#pragma once

#include <stdbool.h>
#include <stddef.h>

struct ufxr_error;

typedef int ufxr_file;

ufxr_file ufxr_file_create(const char *path, struct ufxr_error *err);

bool ufxr_writer_open(struct ufxr_writer *restrict fp, const char *path,
                      struct ufxr_error *err);

bool ufxr_writer_commit(struct ufxr_writer *restrict fp,
                        struct ufxr_error *err);

void ufxr_writer_close(struct ufxr_writer *restrict fp);

bool ufxr_writer_write(struct ufxr_writer *restrict fp, const void *data,
                       size_t size);
