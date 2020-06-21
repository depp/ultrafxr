#include "c/ops/ops.h"
#include "c/util/defs.h"
#include "c/util/flag.h"
#include "c/util/util.h"

#include <errno.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#define EXE_NAME "oprun"

enum {
    kBenchmarkSize = 1 << 20,
    kBenchmarkIter = 1000,
    kBenchmarkRuns = 1,
};

typedef void (*func)(int n, float *restrict outs, const float *restrict xs);

struct func_info {
    char name[8];
    func func;
};

#define F(f) \
    { #f, ufxr_##f }
// clang-format off
static const struct func_info kFuncs[] = {
    F(exp2_2),
    F(exp2_3),
    F(exp2_4),
    F(exp2_5),
    F(exp2_6),
    F(sin1_2),
    F(sin1_3),
    F(sin1_4),
    F(sin1_5),
    F(sin1_6),
    F(tri),
};
// clang-format on
#undef F

static const struct func_info *find_func(const char *name) {
    for (size_t i = 0; i < ARRAY_SIZE(kFuncs); i++) {
        if (strcmp(name, kFuncs[i].name) == 0) {
            return &kFuncs[i];
        }
    }
    die_usagef("unknown function %s", quote_str(name));
}

static void help_benchmark(const char *name) {
    xprintf(stdout, "\nUsage: %s [<pattern>] [<option>...]\n", name);
    xputs(stdout,
          "\n"
          "Options:\n"
          "  -size <size>   Size of input array\n"
          "  -iter <count>  Number of function iterations per run\n"
          "  -runs <count>  Number of benchmark runs\n"
          "  -out <file>    Write results as CSV to <file>\n");
}

static double benchmark(int size, int iter, func f, const float *xs,
                        float *ys) {
    f(size, ys, xs); // Warm cache.
    struct timespec t0, t1;
    clock_gettime(CLOCK_THREAD_CPUTIME_ID, &t0);
    for (int i = 0; i < iter; i++) {
        f(size, ys, xs);
    }
    clock_gettime(CLOCK_THREAD_CPUTIME_ID, &t1);
    return 1e9 * (t1.tv_sec - t0.tv_sec) + (t1.tv_nsec - t0.tv_nsec);
}

static int exec_benchmark(int argc, char **argv) {
    // Parse flags
    int size = kBenchmarkSize;
    int iter = kBenchmarkIter;
    int runs = kBenchmarkRuns;
    const char *outfile = NULL;
    flag_int(&size, "size", "array size");
    flag_int(&iter, "iter", "iteration count");
    flag_int(&runs, "runs", "number of runs");
    flag_string(&outfile, "out", "output file");
    argc = flag_parse(argc, argv);
    bool funcs[ARRAY_SIZE(kFuncs)]; // Which functions to benchmark.
    if (argc == 0) {
        for (size_t func = 0; func < ARRAY_SIZE(kFuncs); func++) {
            funcs[func] = true;
        }
    } else {
        for (size_t func = 0; func < ARRAY_SIZE(kFuncs); func++) {
            funcs[func] = false;
        }
        for (int i = 0; i < argc; i++) {
            char *pat = argv[i];
            char *star = strchr(pat, '*');
            if (star == NULL) {
                bool found = false;
                for (size_t func = 0; func < ARRAY_SIZE(kFuncs); func++) {
                    if (strcmp(kFuncs[func].name, pat) == 0) {
                        found = true;
                        funcs[func] = true;
                        break;
                    }
                }
                if (!found) {
                    die_usagef("unknown function %s", quote_str(pat));
                }
            } else {
                if (*(star + 1) != '\0') {
                    die_usagef("invalid pattern %s, '*' must be at end",
                               quote_str(pat));
                }
                bool found = false;
                for (size_t func = 0; func < ARRAY_SIZE(kFuncs); func++) {
                    if (strncmp(kFuncs[func].name, pat, star - pat) == 0) {
                        found = true;
                        funcs[func] = true;
                    }
                }
                if (!found) {
                    die_usagef("no function matches pattern %s",
                               quote_str(pat));
                }
            }
        }
    }
    if (size < 1) {
        die_usage("size must be positive");
    }
    if ((size % UFXR_QUANTUM) != 0) {
        die_usagef("invalid size %d, must be a multiple of %d", size,
                   UFXR_QUANTUM);
    }
    if (iter < 1) {
        die_usage("iteration count must be positive");
    }
    if (runs < 1) {
        die_usage("run count must be positive");
    }

    // Execute
    float *xs = xmalloc(sizeof(float) * size);
    float *ys = xmalloc(sizeof(float) * size);
    double samples = (double)iter * (double)size;
    linspace(size, xs, -5.0f, 5.0f);
    FILE *fp;
    if (outfile == NULL) {
        fp = stdout;
    } else {
        fp = fopen(outfile, "w");
        if (fp == NULL) {
            int ecode = errno;
            dief(ecode, "could not open %s", quote_str(outfile));
        }
    }
    xputs(fp, "Operator,TimeNS\n");
    for (int run = 0; run < runs; run++) {
        for (size_t func = 0; func < ARRAY_SIZE(kFuncs); func++) {
            if (funcs[func]) {
                double t = benchmark(size, iter, kFuncs[func].func, xs, ys);
                xprintf(fp, "%s,%.3f\n", kFuncs[func].name, t / samples);
            }
        }
    }
    if (outfile != NULL) {
        if (fclose(fp) != 0) {
            int ecode = errno;
            dief(ecode, "error writing to %s", quote_str(outfile));
        }
    }
    return 0;
}

static void help_dump(const char *name) {
    xprintf(stdout, "\nUsage: %s <function> [<min> <max>]\n", name);
    xputs(stdout,
          "\n"
          "Options:\n"
          "  -count <count>  Set number of data points\n"
          "  -out <file>     Write output to <file>\n"
          "  -point-set      Write in macOS Grapher point set format\n");
}

static int exec_dump(int argc, char **argv) {
    int count = 500;
    const char *outfile = NULL;
    bool point_set = false;
    flag_int(&count, "count", "number of data points");
    flag_string(&outfile, "out", "output file");
    flag_bool(&point_set, "point-set", "use point set format");
    argc = flag_parse(argc, argv);
    if (argc < 1) {
        die_usage("missing argument <function>");
    }
    if (count < 1) {
        die_usage("count must be positive");
    }
    const struct func_info *finfo = find_func(argv[0]);
    float x0, x1;
    switch (argc) {
    case 1:
        x0 = -5.0f;
        x1 = 5.0f;
        break;
    case 2:
        die_usage("missing argument <max>");
    case 3:
        x0 = xatof(argv[1]);
        x1 = xatof(argv[2]);
        break;
    default:
        die_usagef("unexpected argument %s", quote_str(argv[3]));
    }
    size_t asize = (count + UFXR_QUANTUM - 1) & ~((size_t)(UFXR_QUANTUM - 1));
    float *xs = xmalloc(asize * sizeof(float));
    float *ys = xmalloc(asize * sizeof(float));
    linspace(count, xs, x0, x1);
    for (size_t i = count; i < asize; i++) {
        xs[i] = 0.0f;
    }
    finfo->func(asize, ys, xs);
    FILE *fp;
    if (outfile == NULL) {
        fp = stdout;
    } else {
        fp = fopen(outfile, "w");
    }
    if (point_set) {
        for (size_t i = 0, n = count; i < n; i++) {
            xprintf(fp, "%zu\t%f\t%f\n", i, (double)xs[i], (double)ys[i]);
        }
    } else {
        xputs(fp, "X,Y\n");
        for (size_t i = 0, n = count; i < n; i++) {
            xprintf(fp, "%f,%f\n", (double)xs[i], (double)ys[i]);
        }
    }
    if (outfile != NULL) {
        if (fclose(fp) != 0) {
            int ecode = errno;
            dief(ecode, "error writing to %s", quote_str(outfile));
        }
    }
    return 0;
}

static void help_help(const char *name);
static int exec_help(int argc, char **argv);

struct cmd_info {
    const char *name;
    const char *desc;
    void (*help)(const char *name);
    int (*exec)(int argc, char **argv);
};

static const struct cmd_info kCmds[] = {
    {"benchmark", "Benchmark functions", help_benchmark, exec_benchmark},
    {"dump", "Dump function output to CSV", help_dump, exec_dump},
    {"help", "Show help", help_help, exec_help},
};

static const struct cmd_info *find_cmd(const char *name) {
    for (size_t i = 0; i < ARRAY_SIZE(kCmds); i++) {
        if (strcmp(name, kCmds[i].name) == 0) {
            return &kCmds[i];
        }
    }
    die_usagef("no command named %s", quote_str(name));
}

static void usage(FILE *fp) {
    xprintf(fp,
            "%s: Execute UltraFXR operators\n"
            "\n"
            "Usage: %s <cmd> [<args>]\n"
            "\n"
            "Commands:\n",
            EXE_NAME, EXE_NAME);
    for (size_t i = 0; i < ARRAY_SIZE(kCmds); i++) {
        xprintf(fp, "  %s: %s\n", kCmds[i].name, kCmds[i].desc);
    }
}

static void help_cmd(const struct cmd_info *cmd) {
    char fullname[64];
    snprintf(fullname, sizeof(fullname), "%s %s", EXE_NAME, cmd->name);
    xprintf(stdout, "%s: %s\n", fullname, cmd->desc);
    cmd->help(fullname);
}

static void help_help(const char *name) {
    xprintf(stdout, "\nUsage: %s [<topic>]\n", name);
}

static int exec_help(int argc, char **argv) {
    if (argc <= 1) {
        usage(stdout);
    } else {
        help_cmd(find_cmd(argv[1]));
    }
    return 0;
}

static bool is_help_flag(const char *arg) {
    return strcmp(arg, "-h") == 0 || strcmp(arg, "-help") == 0 ||
           strcmp(arg, "--help") == 0;
}

int main(int argc, char **argv) {
    if (argc <= 1) {
        usage(stderr);
        return 64;
    }
    argc -= 1;
    argv += 1;
    const char *cmd = argv[0];
    if (is_help_flag(cmd)) {
        usage(stdout);
        return 0;
    }
    const struct cmd_info *info = find_cmd(cmd);
    if (argc >= 2 && is_help_flag(argv[1])) {
        help_cmd(info);
        return 0;
    }
    return info->exec(argc, argv);
}
