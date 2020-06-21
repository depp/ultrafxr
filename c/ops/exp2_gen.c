// exp2_gen.c - Generate exp2 functions.
#include "c/util/util.h"

#include <errno.h>
#include <stdlib.h>
#include <unistd.h>

static void emit(int order, char **coeffs) {
    char fname[30];
    xsprintf(fname, sizeof(fname), "exp2_%d.c", order);
    FILE *fp = fopen(fname, "wb");
    if (fp == NULL) {
        goto error;
    }

    const char *args =
        "(int n, float *restrict outs, const float *restrict xs)";

    xputs(fp, kNotice);
    xputs(fp, "#include \"c/ops/impl.h\"\n");
    xputs(fp,
          "\n"
          "// SSE2 version.\n"
          "#if !HAVE_FUNC && USE_SSE2\n"
          "#define HAVE_FUNC 1\n"
          "#include <xmmintrin.h>\n");
    xprintf(fp, "void ufxr_exp2_%d%s {\n", order, args);
    xputs(fp, "    CHECK2(n, outs, xs);\n");
    for (int i = 0; i <= order; i++) {
        xprintf(fp, "    const __m128 c%d = _mm_set1_ps(%sf);\n", i, coeffs[i]);
    }
    xputs(fp,
          "    for (int i = 0; i < n; i += 4) {\n"
          "        __m128 x = _mm_load_ps(xs + i);\n"
          "        __m128i ival = _mm_cvtps_epi32(x);\n"
          "        __m128 frac = "
          "_mm_sub_ps(x, _mm_cvtepi32_ps(ival));\n");
    xprintf(fp, "        __m128 y = c%d;\n", order);
    for (int i = order - 1; i >= 0; i--) {
        xprintf(fp, "        y = _mm_add_ps(_mm_mul_ps(y, frac), c%d);\n", i);
    }
    xputs(
        fp,
        "        __m128 exp2ival = _mm_castsi128_ps(_mm_add_epi32(\n"
        "            _mm_slli_epi32(ival, 23), _mm_set1_epi32(0x3f800000)));\n"
        "        _mm_store_ps(outs + i, _mm_mul_ps(y, exp2ival));\n"
        "    }\n"
        "}\n"
        "#endif\n");

    xputs(fp,
          "\n"
          "// Scalar version.\n"
          "#if !HAVE_FUNC\n"
          "#include <math.h>\n");
    xprintf(fp, "void ufxr_exp2_%d%s {\n", order, args);
    xputs(fp, "    CHECK2(n, outs, xs);\n");
    for (int i = 0; i <= order; i++) {
        xprintf(fp, "    const float c%d = %sf;\n", i, coeffs[i]);
    }
    xputs(fp,
          "    for (int i = 0; i < n; i++) {\n"
          "        float x = xs[i];\n"
          "        float ival = rintf(x);\n"
          "        float frac = x - ival;\n");
    xprintf(fp, "        float y = c%d;\n", order);
    for (int i = order - 1; i >= 0; i--) {
        xprintf(fp, "        y = y * frac + c%d;\n", i);
    }
    xputs(fp,
          "        outs[i] = scalbnf(y, (int)ival);\n"
          "    }\n"
          "}\n"
          "#endif\n");

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
    if (argc != 4) {
        fputs("Usage: exp2_gen <max-order> <exp2.csv> <out-dir>\n", stderr);
        exit(64);
    }
    int max_order = xatoi(argv[1]);
    const char *inpath = argv[2];
    const char *outdir = argv[3];

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
        if (fields.count != (size_t)order + 2) {
            dief(0, "line %d: found %zu fields, expected %zu", lineno,
                 fields.count, (size_t)order + 2);
        }
        if (order <= max_order) {
            emit(order, fields.strings + 1);
        }
    }

    return 0;
}
