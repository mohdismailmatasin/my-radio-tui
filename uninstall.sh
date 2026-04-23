#!/bin/bash

set -e

if [ -w /usr/local/bin ]; then
    sudo=sudo
else
    sudo=sudo
fi

cargo uninstall my-radio-tui 2>/dev/null || true

rm -f ~/.cargo/bin/my-radio-tui
$sudo rm -f /usr/local/bin/my-radio-tui
$sudo rm -rf /usr/local/bin/playlist

echo "my-radio-tui has been uninstalled."