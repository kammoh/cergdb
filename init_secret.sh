#!/bin/sh
openssl rand -base64 32 > ./SECRET
openssl rand -base64 16 > ./PASSWORD