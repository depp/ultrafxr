// sin1_gen.c - Generate sin1 functions.
#include "c/util/defs.h"
#include "c/util/util.h"

#include <errno.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

enum {
    kAlgoFull,
    kAlgoOdd,
};

static const char kAlgoNames[][8] = {
    [kAlgoFull] = "full",
    [kAlgoOdd] = "odd",
};

static int find_algorithm(const char *name) {
    if (*name != '\0') {
        for (size_t i = 0; i < ARRAY_SIZE(kAlgoNames); i++) {
            if (strcmp(kAlgoNames[i], name) == 0) {
                return i;
            }
        }
    }
    die_usagef("unknown algorithm: %s", quote_str(name));
}

static const char *const kArgs =
    "(int n, float *restrict outs, const float *restrict xs)";

static void emit_full(FILE *fp, int order, char **coeffs) {
    xputs(fp,
          "\n"
          "// Scalar version.\n"
          "#if !HAVE_FUNC\n"
          "#include <math.h>\n");
    xprintf(fp, "void ufxr_sin1_%d%s {\n", order, kArgs);
    xputs(fp, "    assert((n % UFXR_QUANTUM) == 0);\n");
    for (int i = 0; i < order; i++) {
        xprintf(fp, "    const float c%d = %sf;\n", i, coeffs[i]);
    }
    xputs(fp,
          "    for (int i = 0; i < n; i++) {\n"
          "        float x = xs[i];\n"
          "        x -= rintf(x);\n"
          "        float t1 = 0.5f - x;\n"
          "        float t2 = -0.5f - x;\n"
          "        if (t1 < x)\n"
          "            x = t1;\n"
          "        if (t2 > x)\n"
          "            x = t2;\n"
          "        float ax = fabsf(x);\n");
    xprintf(fp, "        float y = c%d;\n", order - 1);
    for (int i = order - 2; i >= 0; i--) {
        xprintf(fp, "        y = y * ax + c%d;\n", i);
    }
    xputs(fp,
          "        outs[i] = x * y;\n"
          "    }\n"
          "}\n"
          "#endif\n");
}

static void emit_odd(FILE *fp, int order, char **coeffs) {
    xputs(fp,
          "\n"
          "// Scalar version.\n"
          "#if !HAVE_FUNC\n"
          "#include <math.h>\n");
    xprintf(fp, "void ufxr_sin1_%d%s {\n", order, kArgs);
    xputs(fp, "    assert((n % UFXR_QUANTUM) == 0);\n");
    for (int i = 0; i < order - 1; i++) {
        xprintf(fp, "    const float c%d = %sf;\n", i, coeffs[i]);
    }
    xputs(fp,
          "    for (int i = 0; i < n; i++) {\n"
          "        float x = xs[i];\n"
          "        x -= rintf(x);\n"
          "        float t1 = 0.5f - x;\n"
          "        float t2 = -0.5f - x;\n"
          "        if (t1 < x)\n"
          "            x = t1;\n"
          "        if (t2 > x)\n"
          "            x = t2;\n"
          "        float x2 = x * x;\n");
    xprintf(fp, "        float y = c%d;\n", order - 2);
    for (int i = order - 3; i >= 0; i--) {
        xprintf(fp, "        y = y * x2 + c%d;\n", i);
    }
    xputs(fp,
          "        outs[i] = x * y;\n"
          "    }\n"
          "}\n"
          "#endif\n");
}

static void emit(int algorithm, int order, char **coeffs) {
    char fname[30];
    xsprintf(fname, sizeof(fname), "sin1_%d.c", order);
    FILE *fp = fopen(fname, "wb");
    if (fp == NULL) {
        goto error;
    }

    xputs(fp, kNotice);
    xputs(fp,
          "#include \"c/ops/ops.h\"\n"
          "#include <assert.h>\n");

    switch (algorithm) {
    case kAlgoFull:
        emit_full(fp, order, coeffs);
        break;
    case kAlgoOdd:
        emit_odd(fp, order, coeffs);
        break;
    default:
        die(0, "invalid algorithm");
    }

    int r = fclose(fp);
    if (r != 0) {
        goto error;
    }
    return;
error:;
    int ecode = errno;
    dief(ecode, "could not write %s", quote_str(fname));
}

int main(int argc, char **argv) {
    if (argc != 5) {
        fputs(
            "Usage: sin1_gen <algorithm> <max-order> <coeffs.csv> <out-dir>\n",
            stderr);
        exit(64);
    }
    int algorithm = find_algorithm(argv[1]);
    int max_order = xatoi(argv[2]);
    const char *inpath = argv[3];
    const char *outdir = argv[4];

    struct data data = {0};
    read_file(&data, inpath);
    int r = chdir(outdir);
    if (r != 0) {
        die(errno, "chdir");
    }
    struct strings lines = {0};
    split_lines(&lines, &data);
    struct strings fields = {0};
    for (size_t lineidx = 0; lineidx < lines.count; lineidx++) {
        int lineno = lineidx + 1;
        char *line = lines.strings[lineidx];
        if (*line == '\0') {
            continue;
        }
        split_csv(&fields, line);
        char *ostr = fields.strings[0], *end;
        long order = strtol(ostr, &end, 10);
        if (*ostr == '\0' || *end != '\0' || order < 0) {
            dief(0, "line %d: invalid order: %s", lineno, quote_str(ostr));
        }
        size_t nfields;
        switch (algorithm) {
        case kAlgoFull:
            nfields = order + 1;
            break;
        case kAlgoOdd:
            // CSV file uses different idea of what "order" means.
            order++;
            nfields = order;
            break;
        default:
            die(0, "unknown algorithm");
        }
        if (order < 3 || order > max_order) {
            continue;
        }
        if (fields.count != nfields) {
            dief(0, "line %d: found %zu fields, expected %zu", lineno,
                 fields.count, nfields);
        }
        emit(algorithm, order, fields.strings + 1);
    }

    return 0;
}
