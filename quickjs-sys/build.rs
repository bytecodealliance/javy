fn main() {
    cc::Build::new()
        .files(&[
               "quickjs/cutils.c",
               "quickjs/libbf.c",
               "quickjs/libregexp.c",
               "quickjs/libunicode.c",
               "quickjs/quickjs.c",
        ])
        .define("_GNU_SOURCE", None)
        .define(
            "CONFIG_VERSION",
            "\"2021-03-27\"",
            )
        .define("CONFIG_BIGNUM", None)
        .cargo_metadata(true)
        // The below flags are used by the official Makefile.
        .flag_if_supported("-Wchar-subscripts")
        .flag_if_supported("-Wno-array-bounds")
        .flag_if_supported("-Wno-format-truncation")
        .flag_if_supported("-Wno-missing-field-initializers")
        .flag_if_supported("-Wno-sign-compare")
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wundef")
        .flag_if_supported("-Wuninitialized")
        .flag_if_supported("-Wunused")
        .flag_if_supported("-Wwrite-strings")
        .flag_if_supported("-funsigned-char")
        .flag_if_supported("-Wno-cast-function-type")
        .flag_if_supported("-Wno-implicit-fallthrough")
        .flag_if_supported("-Wno-enum-conversion")
        .opt_level(2)
        .compile("quickjs");

}

