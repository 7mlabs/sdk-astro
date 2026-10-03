#ifndef ASTRO_ENGINE_H
#define ASTRO_ENGINE_H
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif
uint32_t astro_abi_version(void);
/* UTF-8 JSON input, NUL-terminated. Output is owned; free exactly once below. */
char *astro_calculate_json(const char *input);
/* Separate JSON->JSON context preparation. Input: {"payload": envelope, "options": {...}}. */
char *astro_compress_json(const char *input);
/* Decode an astro-context/1 envelope to its retained engine envelope. */
char *astro_expand_context_json(const char *input);
void astro_free_string(char *output);
#ifdef __cplusplus
}
#endif
#endif
