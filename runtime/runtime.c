/*
 * Copyright (c) 2026 Ztry8 (AslanD)
 * Licensed under the Apache License, Version 2.0 (the "License");
 * You may obtain a copy of the License at http://www.apache.org/licenses/LICENSE-2.0
 */

#if !defined(_WIN32)
#define _POSIX_C_SOURCE 200112L
#endif

#include "runtime.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <time.h>

#if defined(_WIN32)
#include <windows.h>
#endif

void cyl_panic(const char *msg)
{
    fprintf(stderr, "Runtime error: %s\n", msg);
    exit(1);
}

static void *cyl_xmalloc(size_t n)
{
    void *p;
    if (n == 0) {
        n = 1;
    }
    p = malloc(n);
    if (p == NULL) {
        cyl_panic("out of memory");
    }
    return p;
}

CylString cyl_string_from_literal(const char *lit, size_t len)
{
    CylString s;
    s.data = (char *)cyl_xmalloc(len + 1);
    memcpy(s.data, lit, len);
    s.data[len] = '\0';
    s.len = len;
    return s;
}

void cyl_string_free(CylString s)
{
    free(s.data);
}

CylString cyl_string_clone(CylString s)
{
    return cyl_string_from_literal(s.data, s.len);
}

static CylString cyl_string_concat_raw(const char *a, size_t alen,
                                        const char *b, size_t blen)
{
    CylString out;
    out.len = alen + blen;
    out.data = (char *)cyl_xmalloc(out.len + 1);
    memcpy(out.data, a, alen);
    memcpy(out.data + alen, b, blen);
    out.data[out.len] = '\0';
    return out;
}

CylString cyl_string_concat(CylString a, CylString b)
{
    CylString out = cyl_string_concat_raw(a.data, a.len, b.data, b.len);
    cyl_string_free(a);
    cyl_string_free(b);
    return out;
}

CylString cyl_string_concat_int(CylString a, cyl_int b)
{
    CylString bs = cyl_int_to_string(b);
    CylString out = cyl_string_concat_raw(a.data, a.len, bs.data, bs.len);
    cyl_string_free(a);
    cyl_string_free(bs);
    return out;
}

CylString cyl_string_concat_float(CylString a, double b)
{
    CylString bs = cyl_float_to_string(b);
    CylString out = cyl_string_concat_raw(a.data, a.len, bs.data, bs.len);
    cyl_string_free(a);
    cyl_string_free(bs);
    return out;
}

CylString cyl_string_concat_int_rev(cyl_int a, CylString b)
{
    CylString as = cyl_int_to_string(a);
    CylString out = cyl_string_concat_raw(as.data, as.len, b.data, b.len);
    cyl_string_free(as);
    cyl_string_free(b);
    return out;
}

CylString cyl_string_concat_float_rev(double a, CylString b)
{
    CylString as = cyl_float_to_string(a);
    CylString out = cyl_string_concat_raw(as.data, as.len, b.data, b.len);
    cyl_string_free(as);
    cyl_string_free(b);
    return out;
}

CylString cyl_string_repeat(CylString s, cyl_int n)
{
    CylString out;
    size_t i;
    size_t count = (n > 0) ? (size_t)n : 0;

    out.len = s.len * count;
    out.data = (char *)cyl_xmalloc(out.len + 1);
    for (i = 0; i < count; i++) {
        memcpy(out.data + i * s.len, s.data, s.len);
    }
    out.data[out.len] = '\0';

    cyl_string_free(s);
    return out;
}

cyl_bool cyl_string_eq(CylString a, CylString b)
{
    cyl_bool r = (a.len == b.len) && (memcmp(a.data, b.data, a.len) == 0);
    cyl_string_free(a);
    cyl_string_free(b);
    return r;
}

cyl_bool cyl_string_ne(CylString a, CylString b)
{
    return !cyl_string_eq(a, b);
}

static int cyl_string_cmp(CylString a, CylString b)
{
    size_t min_len = (a.len < b.len) ? a.len : b.len;
    int c = memcmp(a.data, b.data, min_len);
    if (c != 0) {
        return c;
    }
    if (a.len < b.len) return -1;
    if (a.len > b.len) return 1;
    return 0;
}

cyl_bool cyl_string_gt(CylString a, CylString b)
{
    int c = cyl_string_cmp(a, b);
    cyl_string_free(a);
    cyl_string_free(b);
    return c > 0;
}

cyl_bool cyl_string_lt(CylString a, CylString b)
{
    int c = cyl_string_cmp(a, b);
    cyl_string_free(a);
    cyl_string_free(b);
    return c < 0;
}

cyl_bool cyl_string_ge(CylString a, CylString b)
{
    int c = cyl_string_cmp(a, b);
    cyl_string_free(a);
    cyl_string_free(b);
    return c >= 0;
}

cyl_bool cyl_string_le(CylString a, CylString b)
{
    int c = cyl_string_cmp(a, b);
    cyl_string_free(a);
    cyl_string_free(b);
    return c <= 0;
}

CylString cyl_int_to_string(cyl_int v)
{
    char buf[32];
    int n = sprintf(buf, "%ld", v);
    return cyl_string_from_literal(buf, (size_t)n);
}

CylString cyl_float_to_string(double v)
{
    char buf[64];
    int n = sprintf(buf, "%g", v);
    return cyl_string_from_literal(buf, (size_t)n);
}

CylString cyl_bool_to_string(cyl_bool v)
{
    if (v) {
        return cyl_string_from_literal("true", 4);
    }
    return cyl_string_from_literal("false", 5);
}

cyl_int cyl_string_to_int(CylString s)
{
    cyl_int v = (cyl_int)strtol(s.data, NULL, 10);
    cyl_string_free(s);
    return v;
}

double cyl_string_to_float(CylString s)
{
    double v = strtod(s.data, NULL);
    cyl_string_free(s);
    return v;
}

cyl_bool cyl_string_to_bool(CylString s)
{
    cyl_bool v = (strcmp(s.data, "true") == 0);
    cyl_string_free(s);
    return v;
}

CylArrayInt cyl_array_int_new(size_t n)
{
    CylArrayInt a;
    a.len = n;
    a.data = (cyl_int *)cyl_xmalloc(n * sizeof(cyl_int));
    return a;
}

CylArrayFloat cyl_array_float_new(size_t n)
{
    CylArrayFloat a;
    a.len = n;
    a.data = (double *)cyl_xmalloc(n * sizeof(double));
    return a;
}

CylArrayBool cyl_array_bool_new(size_t n)
{
    CylArrayBool a;
    a.len = n;
    a.data = (cyl_bool *)cyl_xmalloc(n * sizeof(cyl_bool));
    return a;
}

CylArrayInt cyl_array_int_fill(cyl_int n, cyl_int value)
{
    size_t i;
    CylArrayInt a = cyl_array_int_new((size_t)n);
    for (i = 0; i < a.len; i++) {
        a.data[i] = value;
    }
    return a;
}

CylArrayFloat cyl_array_float_fill(cyl_int n, double value)
{
    size_t i;
    CylArrayFloat a = cyl_array_float_new((size_t)n);
    for (i = 0; i < a.len; i++) {
        a.data[i] = value;
    }
    return a;
}

CylArrayBool cyl_array_bool_fill(cyl_int n, cyl_bool value)
{
    size_t i;
    CylArrayBool a = cyl_array_bool_new((size_t)n);
    for (i = 0; i < a.len; i++) {
        a.data[i] = value;
    }
    return a;
}

cyl_int cyl_array_int_get(CylArrayInt a, cyl_int idx)
{
    if (idx < 0 || (size_t)idx >= a.len) {
        cyl_panic("array index out of bounds");
    }
    return a.data[idx];
}

double cyl_array_float_get(CylArrayFloat a, cyl_int idx)
{
    if (idx < 0 || (size_t)idx >= a.len) {
        cyl_panic("array index out of bounds");
    }
    return a.data[idx];
}

cyl_bool cyl_array_bool_get(CylArrayBool a, cyl_int idx)
{
    if (idx < 0 || (size_t)idx >= a.len) {
        cyl_panic("array index out of bounds");
    }
    return a.data[idx];
}

void cyl_array_int_set(CylArrayInt a, cyl_int idx, cyl_int value)
{
    if (idx < 0 || (size_t)idx >= a.len) {
        cyl_panic("array index out of bounds");
    }
    a.data[idx] = value;
}

void cyl_array_float_set(CylArrayFloat a, cyl_int idx, double value)
{
    if (idx < 0 || (size_t)idx >= a.len) {
        cyl_panic("array index out of bounds");
    }
    a.data[idx] = value;
}

void cyl_array_bool_set(CylArrayBool a, cyl_int idx, cyl_bool value)
{
    if (idx < 0 || (size_t)idx >= a.len) {
        cyl_panic("array index out of bounds");
    }
    a.data[idx] = value;
}

void cyl_array_int_free(CylArrayInt a)
{
    free(a.data);
}

void cyl_array_float_free(CylArrayFloat a)
{
    free(a.data);
}

void cyl_array_bool_free(CylArrayBool a)
{
    free(a.data);
}

size_t cyl_array_int_len(CylArrayInt a)     { return a.len; }
size_t cyl_array_float_len(CylArrayFloat a) { return a.len; }
size_t cyl_array_bool_len(CylArrayBool a)   { return a.len; }

CylString cyl_builtin_input(void)
{
    char buf[4096];
    size_t len;

    if (fgets(buf, sizeof(buf), stdin) == NULL) {
        return cyl_string_from_literal("", 0);
    }

    len = strlen(buf);
    while (len > 0 && (buf[len - 1] == '\n' || buf[len - 1] == '\r')) {
        len--;
    }

    return cyl_string_from_literal(buf, len);
}

double cyl_builtin_sin(double x) { return sin(x); }
double cyl_builtin_cos(double x) { return cos(x); }

double cyl_builtin_sqrt(double x)
{
    if (x < 0.0) {
        cyl_panic("cannot root a negative number");
    }
    return sqrt(x);
}

cyl_int cyl_builtin_shell(CylString cmd)
{
    int code = system(cmd.data);
    cyl_string_free(cmd);
    return (cyl_int)code;
}

cyl_int cyl_builtin_unix_time(void)
{
    return (cyl_int)time(NULL);
}

void cyl_builtin_sleep(cyl_int millis)
{
#if defined(_WIN32)
    Sleep((DWORD)millis);
#else
    struct timespec ts;
    ts.tv_sec = millis / 1000;
    ts.tv_nsec = (millis % 1000) * 1000000;
    nanosleep(&ts, NULL);
#endif
}

void cyl_echo_int(cyl_int v)     { printf("%ld\n", v); }
void cyl_echo_float(double v)    { printf("%g\n", v); }
void cyl_echo_bool(cyl_bool v)   { printf("%s\n", v ? "true" : "false"); }

void cyl_echo_and_free_string(CylString v)
{
    printf("%s\n", v.data);
    cyl_string_free(v);
}

cyl_int cyl_checked_div(cyl_int a, cyl_int b)
{
    if (b == 0) {
        cyl_panic("cannot divide a number by zero");
    }
    return a / b;
}

cyl_int cyl_checked_mod(cyl_int a, cyl_int b)
{
    if (b == 0) {
        cyl_panic("cannot divide a number by zero");
    }
    return a % b;
}
