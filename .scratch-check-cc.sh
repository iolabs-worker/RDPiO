#!/usr/bin/env bash
for c in cc gcc clang; do
  if command -v "$c" >/dev/null 2>&1; then
    echo "FOUND $c: $("$c" --version 2>&1 | head -1)"
  else
    echo "MISSING $c"
  fi
done
command -v ld || echo "MISSING ld"
command -v apt-get && echo "apt-get available" || echo "apt-get MISSING"
command -v apk && echo "apk available" || echo "apk MISSING"
command -v dnf && echo "dnf available" || echo "dnf MISSING"
