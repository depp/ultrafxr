load("//c:copts.bzl", "COPTS")

CORE_COPTS = COPTS + select({
    "//c/config:scalar": ["-DUSE_SCALAR"],
    "//conditions:default": [],
})
