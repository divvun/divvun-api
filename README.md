# Divvun API

Starts a web server for accessing the Divvun spellcheck API.

See https://divvun.github.io/divvun-api/docs/index.html for installation and usage documentation instructions.

## OpenAPI

The OpenAPI documentation is generated with [ReDoc](https://github.com/Redocly/redoc) and hosted at https://divvun.github.io/divvun-api/docs/redoc-static.html

To refresh the documentation, install the [redoc-cli](https://github.com/Redocly/redoc/blob/master/cli/README.md) NPM package and run `redoc-cli bundle openapi.yml`.
This will generate a `redoc-static.html` file that needs to be placed in the `docs` folder.

## Testing

Tests use the files in `tests/resources/data_files`. The files need to be organized as follows before running `cargo test`:

```
tests
|--resources
   |--data_files
      |  se.zcheck
      |  se.zhfst
      |  smj.zcheck
      |  smj.zhfst
      |
      |--grammar
         |  se.zcheck
      |--hyphenation
         |  se.hfstol
      |--spelling
         |  se.zhfst
```

The base `data_files` folder is expected to have both `se` and `smj`
grammar (`.zcheck`) and checker (`.zhfst`) files for the purposes of testing the file watcher, and
the `se` files are also expected to be present in the `spelling`, `hyphenation`, and `grammar` folders for testing loading of files at startup.

- run `cargo test`

## Deployment

Additional steps for deployment.

### Requirements

- Create a regular user with sudo privileges (default: *ubuntu*)
- Create an API user with which the API will run (default: *api*)
- Setup SSH access
- Install python

Set the `admin_email` variable to receive emails from Let's Encrypt when it's time to renew the HTTPS certificate.
