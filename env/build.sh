#!/bin/bash

set -Eeux

podman build -f Containerfile -t snakersdev ctx
