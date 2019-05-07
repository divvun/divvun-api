# Divvun API

Starts a web server for accessing the Divvun spellcheck API.

## Installing

For personal development and testing.

### Prerequisites

- nightly Rust
- development package of OpenSSL
- `divvun-checker` from https://github.com/divvun/libdivvun

Language files need to be placed in these folders:

Linux: `/home/<username>/.local/share/api-giellalt`

Mac OS: `/Users/<username>/Library/Application Support/no.uit.api-giellalt`

Windows: `C:\Users\<username>\AppData\Local\uit\api-giellalt\data`

Inside the folder place `.zcheck` files into `grammar/` and `.zhfst` files into `spelling/` folders.

## Deployment

Additional steps for deployment.

### Requirements

- Create a regular user with sudo privileges (default: *ubuntu*)
- Create an API user with which the API will run (default: *api*)
- Setup SSH access
- Install python

Set the `admin_email` variable to receive emails from Let's Encrypt when it's time to renew the HTTPS certificate.
