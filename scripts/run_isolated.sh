#!/bin/bash
# Desactivar temporalmente el desvío de proxy para que antigravity-core-zero conecte directo a Google
HTTP_PROXY="" HTTPS_PROXY="" http_proxy="" https_proxy="" ./antigravity-core-zero/target/release/antigravity-core-zero
