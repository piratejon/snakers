#!/bin/bash

set -Eeux

dir_=$(dirname "${BASH_SOURCE[0]}")/..

chmod u+s "${dir_}"
chmod g+s "${dir_}"

podman unshare setfacl -R -m u:1000:rwX "${dir_}"

podman run --rm -it -v "${dir_}:/prj" snakersdev /bin/ash -l

