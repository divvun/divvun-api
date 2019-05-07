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

## Deployment

## Requirements

On the target server you need to have made sure the following things exist:

- Created a regular user with sudo privileges (default: *ubuntu*)
- Created an API user with which the API will run (default: *api*)
- Have SSH access
- The target machine has python installed (e.g. `apt-get install python`)

Set the `admin_email` variable to receive emails from let's encrypt when it's time to renew the HTTPS certificate and such.

Perform the steps mentioned above for installation.
