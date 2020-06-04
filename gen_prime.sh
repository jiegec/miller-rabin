#!/bin/sh
openssl prime -generate -bits 1024 > prime_1024
openssl prime -generate -bits 2048 > prime_2048
openssl prime -generate -bits 4096 > prime_4096