COPTS = [
    "-std=c11",
    # "-D_DEFAULT_SOURCE",
    "-Wall",
    "-Wextra",
    "-Wpointer-arith",
    "-Wwrite-strings",
    "-Wmissing-prototypes",
    "-Wdouble-promotion",
    "-Werror=implicit-function-declaration",
    "-Winit-self",
    "-Wstrict-prototypes",
] + select({
    "//support:linux": ["-D_DEFAULT_SOURCE"],
    "//conditions:default": [],
})
