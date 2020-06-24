#include <stdbool.h>
#include <stddef.h>

struct ufxr_error;

// Audio sample formats.
typedef enum {
    kUFXRFormatUnknown,
    kUFXRFormatU8,
    kUFXRFormatS16,
    kUFXRFormatS24,
    kUFXRFormatF32,
} ufxr_format;

// Metadata for a wave file.
struct ufxr_waveinfo {
    // Sample rate in Hz.
    int samplerate;
    // Number of channels. Must be 1 or 2.
    int channels;
    // Sample format.
    ufxr_format format;
    // Length of WAVE file in blocks. Equal to audio length, in seconds,
    // multiplied by sample rate. This is just a hint and may be incorrect or
    // omitted.
    unsigned length;
};

// A wave file for writing audio.
//
// All fields are private. Do not access them.
struct ufxr_wavewriter {
    int file;
    struct ufxr_waveinfo info;
    size_t count;
    void *buffer;
    size_t buffer_pos;
    size_t buffer_size;
    unsigned samples_written;
    unsigned riff_data_written;
    bool at_start;
};

// Create a wave file for writing. If successful, destroy() must be called to
// release resources.
bool ufxr_wavewriter_create(struct ufxr_wavewriter *restrict w,
                            const char *path,
                            const struct ufxr_waveinfo *restrict info,
                            struct ufxr_error *err);

// Destroy the wave file writer and release any resources. If finish() has not
// been called, then the file may not be a valid wave file. Always succeeds.
void ufxr_wavewriter_destroy(struct ufxr_wavewriter *restrict w);

// Finish writing a wave file.
bool ufxr_wavewriter_finish(struct ufxr_wavewriter *restrict w,
                            struct ufxr_error *err);

// Write audio data to the wave file.
bool ufxr_wavewriter_write(struct ufxr_wavewriter *restrict w,
                           const float *restrict data, size_t count,
                           struct ufxr_error *err);
