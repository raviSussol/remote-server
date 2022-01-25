#!/bin/bash
set -e

# Possible HOST_TAG values:
# linux-x86_64, darwin-x86_64, 32-bit Windows: windows, 64-bit Windows: windows-x86_64
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
  HOST_TAG="linux-x86_64"
elif [[ "$OSTYPE" == "darwin"* ]]; then
  HOST_TAG="darwin-x86_64"
else
  echo "Unsupported host: $OSTYPE"
  exit 1
fi

echo $HOST_TAG