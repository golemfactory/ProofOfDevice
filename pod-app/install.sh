#!/usr/bin/env bash
set -ex

# Get current dir.
CURRENT_DIR="$( pwd )"
# Name of the manifest file
APP_NAME="pod_app"
# Native App installation path
INSTALL_DIR="$HOME/.local/share/pod-app"


# Check if user is root.
if [ "$(whoami)" = "root" ]; then

	# Check if Firefox is 64bit.
	if [ -d "/usr/lib64/mozilla/" ]; then
  	  TARGET_DIR="/usr/lib64/mozilla/native-messaging-hosts"
	else
	  TARGET_DIR="/usr/lib/mozilla/native-messaging-hosts"
	fi

else
  TARGET_DIR="$HOME/.mozilla/native-messaging-hosts"
fi

# Create directory to copy manifest.
mkdir -p "$TARGET_DIR"

# Copy pod_app manifest.
cp "$CURRENT_DIR/assets/$APP_NAME.json" "$TARGET_DIR"

# Set permissions for the manifest so all users can read it.
chmod o+r "$TARGET_DIR/$APP_NAME.json"

# Update app path in the manifest.
ESCAPED_APP_PATH=${INSTALL_DIR////\\/}
sed -i -e "s/NATIVE_APP_PATH/$ESCAPED_APP_PATH/" "$TARGET_DIR/$APP_NAME.json"

# Set permissions for the manifest to be shown by all users.
chmod o+r "$TARGET_DIR/$APP_NAME.json"

cargo install --path . --force

if [ -d $INSTALL_DIR ]; then
	rm --recursive $INSTALL_DIR
fi

mkdir -p $INSTALL_DIR

cp -f ../pod-enclave/pod_enclave.signed.so $INSTALL_DIR

echo "$APP_NAME has been installed."
