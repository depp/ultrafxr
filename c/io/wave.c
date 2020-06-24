#include "c/io/wave.h"

#include "c/convert/convert.h"
#include "c/io/error.h"
#include "c/util/defs.h"

#include <assert.h>
#include <errno.h>
#include <fcntl.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

// WAVE formats
//
// WAVE_FORMAT_PCM = 0x0001
//   - Number of bits per sample is rounded up to multiple of 8
//   - Unsigned for <= 8 bits, signed for > 8 bits
//
// WAVE_FORMAT_IEEE_FLOAT = 0x0003

enum {
    kUFXRWaveBufferSize = 16 * 1024,
};

static inline unsigned short le16(unsigned short x);
static inline unsigned le32(unsigned x);

#if __BYTE_ORDER__ && __BYTE_ORDER__ == __ORDER_LITTLE_ENDIAN__
static inline unsigned short le16(unsigned short x) {
    return x;
}
static inline unsigned le32(unsigned x) {
    return x;
}
#elif __BYTE_ORDER__ && __BYTE_ORDER__ == __ORDER_BIG_ENDIAN__
static inline unsigned short le16(unsigned short x) {
    return __builtin_bswap16(x);
}
static inline unsigned le32(unsigned x) {
    return __builtin_bswap32(x);
}
#else
#error "unknown byte order"
#endif

static inline void put16(void *ptr, unsigned short x) {
    unsigned short value = le16(x);
    memcpy(ptr, &value, 2);
}

static inline void put32(void *ptr, unsigned x) {
    unsigned value = le32(x);
    memcpy(ptr, &value, 4);
}

struct ufxr_waveformat {
    unsigned format; // WAVE format (1 = PCM, 3 = float)
    unsigned size;   // Size of sample in bytes
};

static const struct ufxr_waveformat kUFXRWaveFormats[] = {
    [kUFXRFormatU8] = {.format = 1, .size = 1},
    [kUFXRFormatS16] = {.format = 1, .size = 2},
    [kUFXRFormatS24] = {.format = 1, .size = 3},
    [kUFXRFormatF32] = {.format = 3, .size = 4},
};

// Write the WAVE header to the given buffer. Return the number of bytes
// written, or -1 to signal an error.
static int ufxr_wavewriter_writeheader(
    const struct ufxr_waveinfo *restrict info, void *restrict buffer,
    struct ufxr_error *err) {
    if (info->channels < 1 || info->channels > 2) {
        goto inval_arg;
    }
    const struct ufxr_waveformat *fmt;
    if (info->format < 0 ||
        (size_t)info->format > ARRAY_SIZE(kUFXRWaveFormats)) {
        goto inval_arg;
    }
    fmt = &kUFXRWaveFormats[info->format];
    if (fmt->format == 0) {
        goto inval_arg;
    }
    // Non-PCM formats required extended fmt chunk.
    bool extended = fmt->format != 1;
    unsigned fmt_size = extended ? 18 : 16;
    unsigned frame_size = info->channels * fmt->size;
    unsigned data_size;
    if (__builtin_mul_overflow(frame_size, info->length, &data_size)) {
        goto too_long;
    }
    unsigned riff_size;
    if (__builtin_add_overflow(data_size, fmt_size + 20, &riff_size)) {
        goto too_long;
    }

    char *start = buffer;
    char *restrict ptr = start;

    // RIFF header
    memcpy(ptr, "RIFF", 4);     // Identifies as RIFF
    put32(ptr + 4, riff_size);  // Number of bytes after header
    memcpy(ptr + 8, "WAVE", 4); // RIFF type
    ptr += 12;

    // fmt chunk
    // Non-PCM versions require extended format.

    memcpy(ptr + 0, "fmt ", 4); // Chunk type
    put32(ptr + 4, fmt_size);   // Chunk size, in bytes
    ptr += 8;
    put16(ptr + 0, fmt->format);      // WAVE format (1 = PCM, 3 = float)
    put16(ptr + 2, info->channels);   // Number of channels
    put32(ptr + 4, info->samplerate); // Sample rate in Hz
    put32(ptr + 8, info->samplerate * frame_size); // Data rate bytes/sec
    put16(ptr + 12, frame_size);                   // Data block size in bytes
    put16(ptr + 14, fmt->size * 8);                // Bits per sample
    if (extended) {
        put16(ptr + 16, 0);
    }
    ptr += fmt_size;

    // data chunk header
    memcpy(ptr, "data", 4);    // Chunk type
    put32(ptr + 4, data_size); // Chunk size, in bytes

    return fmt_size + 28;

inval_arg:
    ufxr_error_setcode(err, kUFXRErrorInvalidArgument);
    return -1;

too_long:
    ufxr_error_setcode(err, kUFXRErrorTooLong);
    return -1;
}

bool ufxr_wavewriter_create(struct ufxr_wavewriter *restrict w,
                            const char *path,
                            const struct ufxr_waveinfo *restrict info,
                            struct ufxr_error *err) {
    size_t buffersize = kUFXRWaveBufferSize;
    void *buffer = malloc(buffersize);
    if (buffer == NULL) {
        ufxr_error_seterrno(err);
        return false;
    }
    int hlen = ufxr_wavewriter_writeheader(info, buffer, err);
    if (hlen == -1) {
        free(buffer);
        return false;
    }
    int file = open(path, O_WRONLY | O_CREAT | O_TRUNC, 0666);
    if (file == -1) {
        ufxr_error_seterrno(err);
        free(buffer);
        return false;
    }
    *w = (struct ufxr_wavewriter){
        .file = file,
        .info = *info,
        .count = 0,
        .buffer = buffer,
        .buffer_pos = hlen,
        .buffer_size = kUFXRWaveBufferSize,
        .samples_written = 0,
        .riff_data_written = hlen - 8,
        .at_start = true,
    };
    return true;
}

void ufxr_wavewriter_destroy(struct ufxr_wavewriter *restrict w) {
    if (w->file) {
        close(w->file);
        w->file = -1;
    }
    free(w->buffer);
    w->buffer = NULL;
}

static bool ufxr_wavewriter_flush(struct ufxr_wavewriter *restrict w,
                                  struct ufxr_error *err) {
    w->at_start = false;
    char *buf = w->buffer, *end = buf + w->buffer_pos;
    while (buf < end) {
        ssize_t amt = write(w->file, buf, end - buf);
        if (amt == -1) {
            int ecode = errno;
            if (ecode != EINTR) {
                ufxr_error_setsystem(err, ecode);
                return false;
            }
        } else {
            buf += amt;
        }
    }
    return true;
}

static bool ufxr_wavewriter_flushstart(struct ufxr_wavewriter *restrict w,
                                       struct ufxr_error *err) {
    off_t off = 0;
    char *buf = w->buffer, *end = buf + w->buffer_pos;
    while (buf < end) {
        ssize_t amt = pwrite(w->file, buf, end - buf, off);
        if (amt == -1) {
            int ecode = errno;
            if (ecode != EINTR) {
                ufxr_error_setsystem(err, ecode);
                return false;
            }
        } else {
            buf += amt;
            off += amt;
        }
    }
    return true;
}

bool ufxr_wavewriter_finish(struct ufxr_wavewriter *restrict w,
                            struct ufxr_error *err) {
    if (w->file == -1) {
        ufxr_error_setcode(err, kUFXRErrorInvalidArgument);
        return false;
    }
    bool need_header = w->samples_written != w->info.length * w->info.channels;
    if (need_header && w->at_start) {
        int hlen = ufxr_wavewriter_writeheader(&w->info, w->buffer, err);
        if (hlen == -1) {
            return false;
        }
        if (!ufxr_wavewriter_flush(w, err)) {
            return false;
        }
    } else {
        if (!ufxr_wavewriter_flush(w, err)) {
            return false;
        }
        if (need_header) {
            int hlen = ufxr_wavewriter_writeheader(&w->info, w->buffer, err);
            if (hlen == -1) {
                return false;
            }
            w->buffer_pos = hlen;
            if (!ufxr_wavewriter_flushstart(w, err)) {
                return false;
            }
        }
    }
    int r = close(w->file);
    w->file = -1;
    if (r == -1) {
        ufxr_error_seterrno(err);
        return false;
    }
    return true;
}

static bool ufxr_wavewriter_writeu8(struct ufxr_wavewriter *restrict w,
                                    const float *restrict data, size_t count,
                                    struct ufxr_error *err) {
    unsigned riff_data_written;
    if (__builtin_add_overflow(w->riff_data_written, count,
                               &riff_data_written)) {
        ufxr_error_setcode(err, kUFXRErrorTooLong);
        return false;
    }
    w->riff_data_written = riff_data_written;
    w->samples_written += count;
    const float *dpos = data, *dend = dpos + count;
    char *start = w->buffer, *pos = start + w->buffer_pos,
         *end = start + w->buffer_size;
    while (dpos < dend) {
        if (pos == end) {
            w->buffer_pos = end - start;
            if (!ufxr_wavewriter_flush(w, err)) {
                return false;
            }
            pos = start;
        }
        size_t bufrem = end - pos;
        size_t datarem = dend - dpos;
        size_t n = bufrem < datarem ? bufrem : datarem;
        ufxr_to_u8(n, pos, dpos);
        pos += n;
        dpos += n;
    }
    w->buffer_pos = pos - start;
    return true;
}

static bool ufxr_wavewriter_writes16(struct ufxr_wavewriter *restrict w,
                                     const float *restrict data, size_t count,
                                     struct ufxr_error *err) {
    unsigned riff_data_written;
    if (__builtin_add_overflow(w->riff_data_written, count * 2,
                               &riff_data_written)) {
        ufxr_error_setcode(err, kUFXRErrorTooLong);
        return false;
    }
    w->riff_data_written = riff_data_written;
    w->samples_written += count;
    const float *dpos = data, *dend = dpos + count;
    char *start = w->buffer, *pos = start + w->buffer_pos,
         *end = start + w->buffer_size;
    while (dpos < dend) {
        if (pos == end) {
            w->buffer_pos = end - start;
            if (!ufxr_wavewriter_flush(w, err)) {
                return false;
            }
            pos = start;
        }
        size_t bufrem = end - pos;
        size_t datarem = dend - dpos;
        size_t n = bufrem / 2;
        // This should always be true, because we will always be at an even
        // position in the buffer.
        assert(n > 0);
        if (n > datarem) {
            n = datarem;
        }
        ufxr_to_les16(n, pos, dpos);
        pos += n * 2;
        dpos += n;
    }
    w->buffer_pos = pos - start;
    return true;
}

static bool ufxr_wavewriter_writes24(struct ufxr_wavewriter *restrict w,
                                     const float *restrict data, size_t count,
                                     struct ufxr_error *err) {
    unsigned riff_data_written;
    if (__builtin_add_overflow(w->riff_data_written, count * 3,
                               &riff_data_written)) {
        ufxr_error_setcode(err, kUFXRErrorTooLong);
        return false;
    }
    w->riff_data_written = riff_data_written;
    w->samples_written += count;
    const float *dpos = data, *dend = dpos + count;
    char *start = w->buffer, *pos = start + w->buffer_pos,
         *end = start + w->buffer_size;
    while (dpos < dend) {
        if (pos == end) {
            w->buffer_pos = end - start;
            if (!ufxr_wavewriter_flush(w, err)) {
                return false;
            }
            pos = start;
        }
        size_t bufrem = end - pos;
        size_t datarem = dend - dpos;
        size_t n = bufrem / 3;
        if (n > datarem) {
            n = datarem;
        }
        ufxr_to_les24(n, pos, dpos);
        pos += n * 3;
        dpos += n;
        if (dpos < dend && pos < end) {
            bufrem = end - pos;
            char sdata[3];
            ufxr_to_les24(1, sdata, dpos);
            dpos += 1;
            memcpy(pos, sdata, bufrem);
            w->buffer_pos = end - start;
            if (!ufxr_wavewriter_flush(w, err)) {
                return false;
            }
            pos = start;
            memcpy(pos, sdata + bufrem, sizeof(sdata) - bufrem);
            pos += sizeof(sdata) - bufrem;
        }
    }
    w->buffer_pos = pos - start;
    return true;
}

static bool ufxr_wavewriter_writef32(struct ufxr_wavewriter *restrict w,
                                     const float *restrict data, size_t count,
                                     struct ufxr_error *err) {
    unsigned riff_data_written;
    if (__builtin_add_overflow(w->riff_data_written, count * 4,
                               &riff_data_written)) {
        ufxr_error_setcode(err, kUFXRErrorTooLong);
        return false;
    }
    w->riff_data_written = riff_data_written;
    w->samples_written += count;
    const float *dpos = data, *dend = dpos + count;
    char *start = w->buffer, *pos = start + w->buffer_pos,
         *end = start + w->buffer_size;
    while (dpos < dend) {
        if (pos == end) {
            w->buffer_pos = end - start;
            if (!ufxr_wavewriter_flush(w, err)) {
                return false;
            }
            pos = start;
        }
        size_t bufrem = end - pos;
        size_t datarem = dend - dpos;
        size_t n = bufrem / 4;
        if (n > datarem) {
            n = datarem;
        }
        ufxr_to_lef32(n, pos, dpos);
        pos += n * 4;
        dpos += n;
        if (dpos < dend && pos < end) {
            bufrem = end - pos;
            char sdata[4];
            ufxr_to_lef32(1, sdata, dpos);
            dpos += 1;
            memcpy(pos, sdata, bufrem);
            w->buffer_pos = end - start;
            if (!ufxr_wavewriter_flush(w, err)) {
                return false;
            }
            pos = start;
            memcpy(pos, sdata + bufrem, sizeof(sdata) - bufrem);
            pos += sizeof(sdata) - bufrem;
        }
    }
    w->buffer_pos = pos - start;
    return true;
}

bool ufxr_wavewriter_write(struct ufxr_wavewriter *restrict w,
                           const float *restrict data, size_t count,
                           struct ufxr_error *err) {
    if (w->file == -1) {
        ufxr_error_setcode(err, kUFXRErrorInvalidArgument);
        return false;
    }
    switch (w->info.format) {
    case kUFXRFormatU8:
        return ufxr_wavewriter_writeu8(w, data, count, err);
    case kUFXRFormatS16:
        return ufxr_wavewriter_writes16(w, data, count, err);
    case kUFXRFormatS24:
        return ufxr_wavewriter_writes24(w, data, count, err);
    case kUFXRFormatF32:
        return ufxr_wavewriter_writef32(w, data, count, err);
    default:
        ufxr_error_setcode(err, kUFXRErrorInvalidArgument);
        return false;
    }
}
