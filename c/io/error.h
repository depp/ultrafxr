#pragma once

typedef enum {
    kUFXRDomainNone,
    kUFXRDomainLibrary,
    kUFXRDomainSystem,
} ufxr_errdomain;

typedef enum {
    kUFXRErrorNone,
    // Invalid function argument.
    kUFXRErrorInvalidArgument,
    // Too many samples (cannot write a file this long).
    kUFXRErrorTooLong,
} ufxr_errcode;

struct ufxr_error {
    ufxr_errdomain domain;
    int code;
};

inline void ufxr_error_setcode(struct ufxr_error *err, ufxr_errcode code) {
    *err = (struct ufxr_error){
        .domain = kUFXRDomainLibrary,
        .code = code,
    };
}

inline void ufxr_error_setsystem(struct ufxr_error *err, int code) {
    *err = (struct ufxr_error){
        .domain = kUFXRDomainSystem,
        .code = code,
    };
}

void ufxr_error_seterrno(struct ufxr_error *err);
