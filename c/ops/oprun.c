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

static const char *const kBenchmarkOptions =
    "  -size <size>   Size of input array\n"
    "  -iter <count>  Number of function iterations per run\n"
    "  -runs <count>  Number of benchmark runs\n";

static void help_benchmark(const char *name) {
    xprintf(stdout, "\nUsage: %s <function> [<option>...]\n", name);
    xputs(stdout, "\nOptions:\n");
    xputs(stdout, kBenchmarkOptions);
}

static double benchmark(int size, int iter, func f, const float *xs,
                        float *ys) {
    struct timespec t0, t1;
    clock_gettime(CLOCK_MONOTONIC, &t0);
    for (int i = 0; i < iter; i++) {
        f(size, ys, xs);
    }
    clock_gettime(CLOCK_MONOTONIC, &t1);
    return 1e9 * (t1.tv_sec - t0.tv_sec) + (t1.tv_nsec - t0.tv_nsec);
}

static int exec_benchmark_base(bool all, int argc, char **argv) {
    // Parse flags
    int size = kBenchmarkSize;
    int iter = kBenchmarkIter;
    int runs = kBenchmarkRuns;
    const char *outfile = NULL;
    flag_int(&size, "size", "array size");
    flag_int(&iter, "iter", "iteration count");
    flag_int(&runs, "runs", "number of runs");
    const struct func_info *finfo;
    if (all) {
        flag_string(&outfile, "out", "output file");
        argc = flag_parse(argc, argv);
        finfo = NULL;
        if (argc > 0) {
            die_usagef("unexpected argument %s", quote_str(argv[0]));
        }
    } else {
        argc = flag_parse(argc, argv);
        if (argc < 1) {
            die_usage("missing argument <function>");
        } else if (argc > 1) {
            die_usagef("unexpected argument %s", quote_str(argv[0]));
        }
        finfo = find_func(argv[0]);
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
    double *times = xmalloc(sizeof(double) * runs);
    double samples = (double)iter * (double)size;
    linspace(size, xs, -5.0f, 5.0f);
    if (all) {
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
        for (size_t j = 0; j < ARRAY_SIZE(kFuncs); j++) {
            func f = kFuncs[j].func;
            for (int i = 0; i < runs; i++) {
                double t = benchmark(size, iter, f, xs, ys);
                xprintf(fp, "%s,%.2f\n", kFuncs[j].name, t / samples);
            }
        }
        if (outfile != NULL) {
            if (fclose(fp) != 0) {
                int ecode = errno;
                dief(ecode, "error writing to %s", quote_str(outfile));
            }
        }
    } else {
        func f = finfo->func;
        f(size, ys, xs);
        for (int i = 0; i < runs; i++) {
            double t = benchmark(size, iter, f, xs, ys);
            xprintf(stdout, "Time %d: %.2fns/sample (%.0fns)\n", i, t / samples,
                    t);
            times[i] = t;
        }
    }
    return 0;
}

static int exec_benchmark(int argc, char **argv) {
    return exec_benchmark_base(false, argc, argv);
}

static void help_benchmark_all(const char *name) {
    xprintf(stdout, "\nUsage: %s [<option>...]\n", name);
    xputs(stdout, "\nOptions:\n");
    xputs(stdout, kBenchmarkOptions);
    xputs(stdout, "  -out <file>    Write results as CSV to <file>\n");
}

static int exec_benchmark_all(int argc, char **argv) {
    return exec_benchmark_base(true, argc, argv);
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
    {"benchmark", "Benchmark a function", help_benchmark, exec_benchmark},
    {"benchmark-all", "Benchmark all functions", help_benchmark_all,
     exec_benchmark_all},
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
