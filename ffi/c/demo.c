/* Minimal C consumer of guitarpro-ffi.
 *
 * Reads a score file, reports the detected format, and writes either its JSON
 * representation or a conversion to another format.
 *
 * Build (after `cargo build -p guitarpro-ffi`):
 *
 *     cc -I../include demo.c -L../../target/debug -lguitarpro_ffi \
 *        -Wl,-rpath,../../target/debug -o demo
 *
 * Or simply `just ffi-demo` from the workspace root.
 *
 * Usage:
 *     ./demo <file>            # print the score as JSON
 *     ./demo <file> <format>   # convert; format is gp3|gp4|gp5|gpx|gp|xml|score|mscz
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "guitarpro.h"

static const char *format_name(uint32_t format) {
    switch (format) {
    case GPIO_FORMAT_GP3: return "gp3";
    case GPIO_FORMAT_GP4: return "gp4";
    case GPIO_FORMAT_GP5: return "gp5";
    case GPIO_FORMAT_GPX: return "gpx";
    case GPIO_FORMAT_GP: return "gp";
    case GPIO_FORMAT_MUSIC_XML: return "xml";
    case GPIO_FORMAT_SCORE: return "score";
    case GPIO_FORMAT_MSCZ: return "mscz";
    default: return "unknown";
    }
}

/* Returns GPIO_FORMAT_AUTO when the name is not recognised. */
static uint32_t format_from_name(const char *name) {
    if (strcmp(name, "gp3") == 0) return GPIO_FORMAT_GP3;
    if (strcmp(name, "gp4") == 0) return GPIO_FORMAT_GP4;
    if (strcmp(name, "gp5") == 0) return GPIO_FORMAT_GP5;
    if (strcmp(name, "gpx") == 0) return GPIO_FORMAT_GPX;
    if (strcmp(name, "gp") == 0) return GPIO_FORMAT_GP;
    if (strcmp(name, "xml") == 0) return GPIO_FORMAT_MUSIC_XML;
    if (strcmp(name, "score") == 0) return GPIO_FORMAT_SCORE;
    if (strcmp(name, "mscz") == 0) return GPIO_FORMAT_MSCZ;
    return GPIO_FORMAT_AUTO;
}

/* Reads the whole file into a malloc'd buffer. Caller frees. */
static unsigned char *read_file(const char *path, size_t *out_len) {
    FILE *handle = fopen(path, "rb");
    if (!handle) {
        return NULL;
    }
    if (fseek(handle, 0, SEEK_END) != 0) {
        fclose(handle);
        return NULL;
    }
    long size = ftell(handle);
    if (size < 0) {
        fclose(handle);
        return NULL;
    }
    rewind(handle);

    unsigned char *bytes = malloc((size_t)size);
    if (!bytes) {
        fclose(handle);
        return NULL;
    }
    size_t read = fread(bytes, 1, (size_t)size, handle);
    fclose(handle);
    if (read != (size_t)size) {
        free(bytes);
        return NULL;
    }
    *out_len = read;
    return bytes;
}

/* Prints the library's pending error, guarding against a NULL message. */
static void report_error(const char *context) {
    const char *message = gpio_last_error();
    fprintf(stderr, "%s: %s\n", context, message ? message : "unknown error");
}

int main(int argc, char **argv) {
    if (argc < 2 || argc > 3) {
        fprintf(stderr, "usage: %s <file> [output-format]\n", argv[0]);
        return 2;
    }

    fprintf(stderr, "guitarpro-ffi %s\n", gpio_version());

    size_t len = 0;
    unsigned char *bytes = read_file(argv[1], &len);
    if (!bytes) {
        fprintf(stderr, "cannot read %s\n", argv[1]);
        return 1;
    }

    /* Parse once so the detected format can be reported. */
    GpioScore *score = gpio_parse(bytes, len, GPIO_FORMAT_AUTO);
    if (!score) {
        report_error("parse failed");
        free(bytes);
        return 1;
    }
    fprintf(stderr, "detected format: %s\n", format_name(gpio_score_format(score)));

    GpioBuffer out;
    if (argc == 3) {
        uint32_t target = format_from_name(argv[2]);
        if (target == GPIO_FORMAT_AUTO) {
            fprintf(stderr, "unknown output format '%s'\n", argv[2]);
            gpio_score_free(score);
            free(bytes);
            return 2;
        }
        out = gpio_convert(bytes, len, GPIO_FORMAT_AUTO, target);
    } else {
        out = gpio_score_to_json(score);
    }

    gpio_score_free(score);
    free(bytes);

    if (!out.data) {
        report_error("conversion failed");
        gpio_buffer_free(out);
        return 1;
    }

    fwrite(out.data, 1, out.len, stdout);
    gpio_buffer_free(out);
    return 0;
}
