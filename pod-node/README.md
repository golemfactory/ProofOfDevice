# Pod-Node

It's a native node module, wraps the pod-library.

# Prerequisites

After everything is ready with SGX _(check "pod-server" folder README)_, be sure you have;

- node (from https://nodejs.org, or preffered https://github.com/nvm-sh/nvm)
- node-gyp (npm install -g node-gyp)

- SPID from Intel
- Put your SPID in `src/module.js`

# Compile

`npm run compile`

# Run _(to test)_

`npm run start`