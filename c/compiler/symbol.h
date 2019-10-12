// symbol.h - Symbols and symbol tables.
#pragma once

#include <stddef.h>
#include <stdint.h>

enum {
    // Maximum length of a symbol.
    SYM_MAXLEN = 100,
};

enum {
    // Out of memory.
    SYM_NOMEM = -1,
    // Symbol is too long.
    SYM_TOOLONG = -2,
};

// An entry in a symbol table.
struct ufxr_syment {
    uint32_t index;  // Symbol index, or 0 if no symbol.
    uint32_t length; // Length of symbol, in bytes (not counting nul).
    char *text;      // Pointer to nul-terminated symbol.
};

// A symbol table, mapping symbols to index values.
struct ufxr_symtab {
    struct ufxr_syment *arr;
    uint32_t count;
    uint32_t alloc;
};

// Add a symbol to the symbol table. Return the symbol index, or a negative
// error code above (SYM_NOMEM or SYM_TOOLONG).
int32_t ufxr_symtab_add(struct ufxr_symtab *restrict tab, const char *text,
                        size_t textlen);
