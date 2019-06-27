# Divvun API

Starts a web server for accessing the Divvun spellcheck API.

## Installing

For personal development and testing.

### Prerequisites

- development package of OpenSSL
- `divvun-checker` from https://github.com/divvun/libdivvun

For development, language files need to be placed in these folders:

Linux: `/home/<username>/.local/share/api-giellalt`

Mac OS: `/Users/<username>/Library/Application Support/no.uit.api-giellalt`

Windows: `C:\Users\<username>\AppData\Local\uit\api-giellalt\data`

Inside the folder place `.zcheck` files into `grammar/` and `.zhfst` files into `spelling/` folders.

## Testing

Tests use the files in tests/resources/data_files. The data_files folder is expected to have both `se` and `smj`
grammar and checker files for the purposes of testing the file watcher.

The `se` files are also expected to be present in the `speller` and `grammar` folders for testing initial loading.

- run `cargo test`

## Deployment

Additional steps for deployment.

### Requirements

- Create a regular user with sudo privileges (default: *ubuntu*)
- Create an API user with which the API will run (default: *api*)
- Setup SSH access
- Install python

Set the `admin_email` variable to receive emails from Let's Encrypt when it's time to renew the HTTPS certificate.
