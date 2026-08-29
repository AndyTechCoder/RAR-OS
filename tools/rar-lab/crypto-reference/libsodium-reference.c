#include <sodium.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define MAX_MESSAGE_BYTES (1024u * 1024u)

static int decode_hex(const char *text, unsigned char *output, size_t expected) {
    size_t length = strlen(text);
    size_t decoded = 0;
    const char *end = NULL;
    if (length != expected * 2u) return -1;
    if (sodium_hex2bin(output, expected, text, length, NULL, &decoded, &end) != 0) return -1;
    return decoded == expected && end == text + length ? 0 : -1;
}

static unsigned char *decode_message(const char *text, size_t *length) {
    size_t chars = strlen(text);
    unsigned char *message;
    const char *end = NULL;
    if ((chars & 1u) != 0 || chars / 2u > MAX_MESSAGE_BYTES) return NULL;
    *length = chars / 2u;
    message = malloc(*length == 0 ? 1u : *length);
    if (message == NULL) return NULL;
    if (sodium_hex2bin(message, *length, text, chars, NULL, length, &end) != 0 || end != text + chars) {
        sodium_memzero(message, *length);
        free(message);
        return NULL;
    }
    return message;
}

static void print_hex(const unsigned char *bytes, size_t length) {
    char *hex = malloc(length * 2u + 1u);
    if (hex == NULL) exit(70);
    sodium_bin2hex(hex, length * 2u + 1u, bytes, length);
    puts(hex);
    sodium_memzero(hex, length * 2u + 1u);
    free(hex);
}

int main(int argc, char **argv) {
    unsigned char seed[crypto_sign_SEEDBYTES];
    unsigned char public_key[crypto_sign_PUBLICKEYBYTES];
    unsigned char secret_key[crypto_sign_SECRETKEYBYTES];
    unsigned char signature[crypto_sign_BYTES];
    unsigned char *message = NULL;
    size_t message_length = 0;
    int result = 70;

    if (sodium_init() < 0) return 70;
    if (argc == 2 && strcmp(argv[1], "--version") == 0) {
        puts("libsodium-reference 1.0.19");
        return 0;
    }
    if (argc == 3 && strcmp(argv[1], "public-key") == 0) {
        if (decode_hex(argv[2], seed, sizeof seed) != 0) goto done;
        if (crypto_sign_seed_keypair(public_key, secret_key, seed) != 0) goto done;
        print_hex(public_key, sizeof public_key);
        result = 0;
        goto done;
    }
    if (argc == 4 && strcmp(argv[1], "sign") == 0) {
        if (decode_hex(argv[2], seed, sizeof seed) != 0) goto done;
        message = decode_message(argv[3], &message_length);
        if (message == NULL) goto done;
        if (crypto_sign_seed_keypair(public_key, secret_key, seed) != 0) goto done;
        if (crypto_sign_detached(signature, NULL, message, (unsigned long long)message_length, secret_key) != 0) goto done;
        print_hex(signature, sizeof signature);
        result = 0;
        goto done;
    }
    if (argc == 5 && strcmp(argv[1], "verify") == 0) {
        if (decode_hex(argv[2], public_key, sizeof public_key) != 0) goto done;
        message = decode_message(argv[3], &message_length);
        if (message == NULL || decode_hex(argv[4], signature, sizeof signature) != 0) goto done;
        if (crypto_sign_verify_detached(signature, message, (unsigned long long)message_length, public_key) == 0) {
            puts("valid");
            result = 0;
        } else {
            puts("invalid");
            result = 1;
        }
        goto done;
    }

done:
    sodium_memzero(seed, sizeof seed);
    sodium_memzero(secret_key, sizeof secret_key);
    sodium_memzero(signature, sizeof signature);
    if (message != NULL) {
        sodium_memzero(message, message_length);
        free(message);
    }
    return result;
}
