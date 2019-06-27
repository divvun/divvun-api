# Divvun API

Starts a web server for accessing the Divvun spellcheck API.

See https://divvun.github.io/divvun-api/docs/index.html for installation and usage documentation instructions.

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
