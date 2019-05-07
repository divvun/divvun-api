# Divvun API

Starts a web server for accessing the Divvun spellcheck API

## Installing

### Prerequisites

- nightly Rust
- development package of OpenSSL
- `divvun-checker` from https://github.com/divvun/libdivvun

Language files need to be placed in these folders or similar:

Linux: `/home/<username>/.local/share/api-giellalt`
Mac OS: `/Users/<username>/Library/Application Support/no.uit.api-giellalt`
Windows: `C:\Users\<username>\AppData\Local\uit\api-giellalt\data`

Inside the appropriate folder place `.zcheck` files into `grammar/` and `.zhfst` files into `spelling/`.
