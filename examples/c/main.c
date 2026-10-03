#include "astro_engine.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
int main(int argc, char **argv) {
    if (astro_abi_version() != 1) return 1;
    if (argc > 1 && (!strcmp(argv[1], "--compress-json") || !strcmp(argv[1], "--expand-context-json"))) {
        size_t size = 0, capacity = 4096, limit = 256u * 1024u * 1024u;
        char *input = malloc(capacity);
        if (!input) return 3;
        int ch;
        while ((ch = getchar()) != EOF) {
            if (ch == 0) { free(input); return 5; }
            if (size >= limit) { free(input); return 4; }
            if (size + 1 >= capacity) {
                capacity *= 2;
                char *grown = realloc(input, capacity);
                if (!grown) { free(input); return 3; }
                input = grown;
            }
            input[size++] = (char)ch;
        }
        input[size] = '\0';
        char *result = !strcmp(argv[1], "--compress-json")
            ? astro_compress_json(input) : astro_expand_context_json(input);
        free(input);
        if (!result) return 2;
        puts(result);
        astro_free_string(result);
        return 0;
    }
    const char *input = argc > 1 ? argv[1] : "{\"operation\":\"natal\",\"utc\":{\"year\":2000,\"month\":1,\"day\":1,\"hour\":12,\"minute\":0},\"location\":{\"latitude\":10.8231,\"longitude\":106.6297}}";
    char *result = astro_calculate_json(input);
    if (!result) return 2;
    puts(result);
    astro_free_string(result);
    return 0;
}
