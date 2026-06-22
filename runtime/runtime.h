/*
 * Copyright (c) 2026 Ztry8 (AslanD)
 * Licensed under the Apache License, Version 2.0 (the "License");
 * You may obtain a copy of the License at http://www.apache.org/licenses/LICENSE-2.0
 */

#ifndef CYL_RUNTIME_H
#define CYL_RUNTIME_H

#include <stddef.h>

typedef int cyl_bool;
#define CYL_TRUE  1
#define CYL_FALSE 0

typedef long cyl_int;

typedef struct {
    char *data; 
    size_t len;
} CylString;

typedef struct {
    cyl_int *data;
    size_t len;
} CylArrayInt;

typedef struct {
    double *data;
    size_t len;
} CylArrayFloat;

typedef struct {
    cyl_bool *data;
    size_t len;
} CylArrayBool;

void cyl_panic(const char *msg);

CylString cyl_string_from_literal(const char *lit, size_t len);
void cyl_string_free(CylString s);

CylString cyl_string_concat(CylString a, CylString b);
CylString cyl_string_concat_int(CylString a, cyl_int b);
CylString cyl_string_concat_float(CylString a, double b);
CylString cyl_string_concat_int_rev(cyl_int a, CylString b);
CylString cyl_string_concat_float_rev(double a, CylString b);
CylString cyl_string_repeat(CylString s, cyl_int n);

cyl_bool cyl_string_eq(CylString a, CylString b);
cyl_bool cyl_string_ne(CylString a, CylString b);
cyl_bool cyl_string_gt(CylString a, CylString b);
cyl_bool cyl_string_lt(CylString a, CylString b);
cyl_bool cyl_string_ge(CylString a, CylString b);
cyl_bool cyl_string_le(CylString a, CylString b);

CylString cyl_int_to_string(cyl_int v);
CylString cyl_float_to_string(double v);
CylString cyl_bool_to_string(cyl_bool v);

cyl_int  cyl_string_to_int(CylString s);
double   cyl_string_to_float(CylString s);
cyl_bool cyl_string_to_bool(CylString s);

CylString cyl_string_clone(CylString s);

CylArrayInt   cyl_array_int_new(size_t n);
CylArrayFloat cyl_array_float_new(size_t n);
CylArrayBool  cyl_array_bool_new(size_t n);

CylArrayInt   cyl_array_int_fill(cyl_int n, cyl_int value);
CylArrayFloat cyl_array_float_fill(cyl_int n, double value);
CylArrayBool  cyl_array_bool_fill(cyl_int n, cyl_bool value);

cyl_int  cyl_array_int_get(CylArrayInt a, cyl_int idx);
double   cyl_array_float_get(CylArrayFloat a, cyl_int idx);
cyl_bool cyl_array_bool_get(CylArrayBool a, cyl_int idx);

void cyl_array_int_set(CylArrayInt a, cyl_int idx, cyl_int value);
void cyl_array_float_set(CylArrayFloat a, cyl_int idx, double value);
void cyl_array_bool_set(CylArrayBool a, cyl_int idx, cyl_bool value);

void cyl_array_int_free(CylArrayInt a);
void cyl_array_float_free(CylArrayFloat a);
void cyl_array_bool_free(CylArrayBool a);

CylString cyl_builtin_input(void);
double    cyl_builtin_sin(double x);
double    cyl_builtin_cos(double x);
double    cyl_builtin_sqrt(double x);
cyl_int   cyl_builtin_shell(CylString cmd);
cyl_int   cyl_builtin_unix_time(void);
void      cyl_builtin_sleep(cyl_int millis);
size_t    cyl_array_int_len(CylArrayInt a);
size_t    cyl_array_float_len(CylArrayFloat a);
size_t    cyl_array_bool_len(CylArrayBool a);

void cyl_echo_int(cyl_int v);
void cyl_echo_float(double v);
void cyl_echo_bool(cyl_bool v);
void cyl_echo_and_free_string(CylString v);

cyl_int cyl_checked_div(cyl_int a, cyl_int b);
cyl_int cyl_checked_mod(cyl_int a, cyl_int b);

#endif
