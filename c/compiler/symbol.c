// symbol.c - Symbols and symbol tables.
#include "c/compiler/symbol.h"

#include "c/compiler/hash.h"

#include <assert.h>
#include <stdlib.h>
#include <string.h>

void ufxr_symtab_destroy(struct ufxr_symtab *restrict tab) {
    struct ufxr_syment *restrict es = tab->arr;
    uint32_t n = tab->alloc;
    for (uint32_t i = 0; i < n; i++) {
        if (es[i].index != 0) {
            free(es[i].text);
        }
    }
    free(es);
}

// Normalize a symbol to lowercase, write the result to out, and return the
// hash.
static uint32_t ufxr_symnorm(char *restrict out, const char *restrict text,
                             size_t len) {
    for (size_t i = 0; i < len; i++) {
        unsigned char c = text[i];
        if ('A' <= c && c <= 'Z') {
            c += 'a' - 'A';
        }
        out[i] = c;
    }
    return ufxr_hash(out, len);
}

static uint32_t round_up_pow2(uint32_t x) {
    x -= 1;
    x |= x >> 16;
    x |= x >> 8;
    x |= x >> 4;
    x |= x >> 2;
    x |= x >> 1;
    x += 1;
    return x;
}

// Transfer values from one hash map to another.
static void ufxr_symtab_xfer(struct ufxr_syment *restrict des, uint32_t dn,
                             const struct ufxr_syment *restrict ses,
                             uint32_t sn) {
    for (uint32_t i = 0; i < dn; i++) {
        des[i].index = 0;
    }
    for (uint32_t i = 0; i < sn; i++) {
        if (ses[i].index != 0) {
            uint32_t length = ses[i].length;
            char *text = ses[i].text;
            uint32_t hash = ufxr_hash(text, length);
            for (uint32_t j = 0;; j++) {
                assert(j < dn);
                uint32_t pos = (hash + j) & (dn - 1);
                if (des[pos].index == 0) {
                    des[pos] = ses[i];
                    break;
                }
            }
        }
    }
}

int32_t ufxr_symtab_add(struct ufxr_symtab *restrict tab, const char *text,
                        size_t textlen) {
    if (textlen > SYM_MAXLEN) {
        return SYM_TOOLONG;
    }
    char norm[SYM_MAXLEN];
    uint32_t hash = ufxr_symnorm(norm, text, textlen);
    struct ufxr_syment *restrict es = tab->arr;
    uint32_t n = tab->alloc;
    uint32_t pos = 0;
    for (uint32_t i = 0; i < n; i++) {
        assert(i < n);
        pos = (i + hash) & (n - 1);
        // Empty entry, symbol not present.
        if (es[pos].index == 0) {
            break;
        }
        // Symbol is present.
        if (es[pos].length == textlen &&
            memcmp(es[pos].text, norm, textlen) == 0) {
            return es[pos].index;
        }
    }
    uint32_t newcount = tab->count + 1;
    uint32_t minsize = newcount + (newcount >> 1);
    if (tab->alloc < minsize) {
        uint32_t newsize = round_up_pow2(minsize);
        struct ufxr_syment *restrict newes = malloc(newsize * sizeof(*newes));
        if (!newes) {
            return SYM_NOMEM;
        }
        ufxr_symtab_xfer(newes, newsize, es, n);
        free(es);
        tab->arr = es = newes;
        tab->alloc = n = newsize;
        for (uint32_t i = 0;; i++) {
            // The hash was just enlarged, so there should be a free entry.
            assert(i < n);
            pos = (i + hash) & (n - 1);
            if (es[pos].index == 0) {
                break;
            }
        }
    }
    char *symtext = malloc(textlen + 1);
    if (symtext == NULL) {
        return SYM_NOMEM;
    }
    memcpy(symtext, norm, textlen);
    symtext[textlen] = '\0';
    es[pos] = (struct ufxr_syment){
        .index = newcount,
        .length = textlen,
        .text = symtext,
    };
    tab->count = newcount;
    return newcount;
}
