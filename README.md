# Divvun API

Starts a web server for accessing the Divvun spellcheck API.

See https://divvun.github.io/divvun-api/index.html for installation and usage documentation instructions.

## OpenAPI

The OpenAPI documentation is generated with [ReDoc](https://github.com/Redocly/redoc) and hosted at  https://divvun.github.io/divvun-api/redoc-static.html

To refresh the documentation, install the [redoc-cli](https://github.com/Redocly/redoc/blob/master/cli/README.md) NPM package (`npm i -g redoc-cli`) and run `redoc-cli bundle openapi.yml`.
This will generate a `redoc-static.html` file that needs to be placed in the `docs` folder.

To refresh `docs/index.html`, `cd docs/` and run `asciidoctor index.adoc`.

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


### Docker image

This project is built and pushed into a docker image. The docker images is deployed by the https://github.com/divvun/divvun-api-deploy/

You need to build the docker image on an x86 machine. An m1 mac won't do.


```
docker build -t divvun/divvun-api:v2 .
```

Now.. this image is not uploaded to a repository. It just sits there. You'll have to pack it yourself. 

### Crimes


Basic litmus tests for spellers and grammar

```sh
$ curl -X POST -H 'Content-Type: application/json' 'https://api-giellalt.uit.no/speller/se' --data '{"text": "pahkat"}'
{"text":"pahkat","results":[{"word":"pahkat","is_correct":false,"suggestions":[{"value":"páhkat","weight":15.301758},{"value":"páhkkat","weight":21.3018},{"value":"dahkat","weight":33.012695},{"value":"háhkat","weight":34.89453},{"value":"ráhkat","weight":38.691406},{"value":"čáhkat","weight":38.79785},{"value":"hahkát","weight":39.896484},{"value":"báhkat","weight":39.89746},{"value":"Ráhkat","weight":40.05078},{"value":"páhka","weight":40.301758}]}]
$ curl -X POST -H 'Content-Type: application/json' 'https://api-giellalt.uit.no/grammar/se' --data '{"text": "Danne lea politijuristtaide eanemus praktihkkalaččat vuogas dan dahkat Čáhcesullos."}'
{"text":"Danne lea politijuristtaide eanemus praktihkkalaččat vuogas dan dahkat Čáhcesullos.","errs":[{"error_text":"politijuristtaide","start_index":10,"end_index":27,"error_code":"typo","description":"Ii leat sátnelisttus","suggestions":["politiijajuristtaide"],"title":"Čállinmeattáhus"},{"error_text":"praktihkkalaččat","start_index":36,"end_index":52,"error_code":"typo","description":"Ii leat sátnelisttus","suggestions":["praktihkalaččat","praktihkalat","praktihkalet","praktihkalit","praktihkalut"],"title":"Čállinmeattáhus"}]}%
```


Pack your image:
```sh
docker save divvun/divvun-api:v2 | gzip > divvun-api-v2.tar.gz
```

Copy your image to the divvun-api server:
```
scp divvun-api-v2.tar.gz root@64.225.76.53:
```

Load your image up
```sh
% ssh root@64.225.76.53
$ gunzip --stdout divvun-api-v2.tar.gz | docker load
8553b91047da: Loading layer [==================================================>]  84.01MB/84.01MB
22050e545130: Loading layer [==================================================>]  22.38MB/22.38MB
16c60edfa394: Loading layer [==================================================>]    215kB/215kB
86ffb3b2b27a: Loading layer [==================================================>]  80.01MB/80.01MB
c642b882d7d8: Loading layer [==================================================>]  1.536kB/1.536kB
0b38c186a861: Loading layer [==================================================>]  16.72MB/16.72MB
75784fda3f36: Loading layer [==================================================>]   2.56kB/2.56kB
Loaded image: divvun/divvun-api:v2
```

The default user is API. Switch to it, go to the deploy directory and change the docker-compose to use your new tag. The restart docker-compose and start tailing the newly started divvun-api container. 

```sh
su api -s /bin/bash
cd /home/api/dist
nano -w docker-compose.yml #change the image to use :v2
docker-compose restart
docker logs -n10 -f dist_divvun_api_1
```

And then you rerun your litmus tests. Make sure to change the languages around so you're not testing.. something that would have worked anyway.
And then you change your grammar packages around. And then you rerun the litmus tests. 

Ultimately, you update the https://github.com/divvun/divvun-api-deploy/ repo and stop doing crimes. 
