#!/bin/bash

set -Eeux

cd "$(dirname "${BASH_SOURCE[0]}")"

podman build -f Containerfile -t snakersdev ctx
