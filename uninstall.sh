#!/bin/bash

set -e

if [ -w /usr/local/bin ] && [ -w /usr/local/share ]; then
    SUDO=""
else
    SUDO="sudo"
fi

cargo uninstall my-radio-tui 2>/dev/null || true

rm -f ~/.cargo/bin/my-radio-tui
$SUDO rm -f /usr/local/bin/my-radio-tui
$SUDO rm -rf /usr/local/share/my-radio-tui

echo "my-radio-tui has been uninstalled."
