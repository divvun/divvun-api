# Deployment to API Server

In this folder (deployment) adjust paths to installation folder in divvun-api.server, then run `./build_artifact.sh` and transfer the resulting dist.tar.gz to the deployment server.

Extract the archive on the server, run `sudo install.sh` and enable/start the divvun-api systemd service.

# Downloading Speller & Grammar Archives

Easiest way right now is to get https://github.com/divvun/divvun-ci-config and download all the archives with:

```bash
sh ./repo/scripts/download_speller.sh https://apertium.projectjj.com/apt/nightly/pool/main/g/giella-sme/ usr/share/voikko/4/se.zcheck se.zcheck
sh ./repo/scripts/download_speller.sh https://apertium.projectjj.com/apt/nightly/pool/main/g/giella-sme/ usr/share/giella/mobilespellers/sme-mobile.zhfst sme.zhfst

sh ./repo/scripts/download_speller.sh https://apertium.projectjj.com/apt/nightly/pool/main/g/giella-sma/ usr/share/voikko/4/sma.zcheck sma.zcheck
sh ./repo/scripts/download_speller.sh https://apertium.projectjj.com/apt/nightly/pool/main/g/giella-sma/ usr/share/giella/mobilespellers/sma-mobile.zhfst sma.zhfst

sh ./repo/scripts/download_speller.sh https://apertium.projectjj.com/apt/nightly/pool/main/g/giella-smn/ usr/share/voikko/4/smn.zcheck smn.zcheck
sh ./repo/scripts/download_speller.sh https://apertium.projectjj.com/apt/nightly/pool/main/g/giella-smn/ usr/share/giella/mobilespellers/smn-mobile.zhfst smn.zhfst

sh ./repo/scripts/download_speller.sh https://apertium.projectjj.com/apt/nightly/pool/main/g/giella-sms/ usr/share/voikko/4/sms.zcheck sms.zcheck
sh ./repo/scripts/download_speller.sh https://apertium.projectjj.com/apt/nightly/pool/main/g/giella-sms/ usr/share/giella/mobilespellers/sms-mobile.zhfst sms.zhfst

sh ./repo/scripts/download_speller.sh https://apertium.projectjj.com/apt/nightly/pool/main/g/giella-smj/ usr/share/voikko/4/smj.zcheck smj.zcheck
sh ./repo/scripts/download_speller.sh https://apertium.projectjj.com/apt/nightly/pool/main/g/giella-smj/ usr/share/giella/mobilespellers/smj-mobile.zhfst smj.zhfst

sh ./repo/scripts/download_speller.sh https://apertium.projectjj.com/apt/nightly/pool/main/g/giella-myv/ usr/share/voikko/4/myv.zcheck myv.zcheck
sh ./repo/scripts/download_speller.sh https://apertium.projectjj.com/apt/nightly/pool/main/g/giella-myv/ usr/share/giella/mobilespellers/myv-mobile.zhfst myv.zhfst

sh ./repo/scripts/download_speller.sh https://apertium.projectjj.com/apt/nightly/pool/main/g/giella-crk/ usr/share/voikko/4/crk.zcheck crk.zcheck
sh ./repo/scripts/download_speller.sh https://apertium.projectjj.com/apt/nightly/pool/main/g/giella-crk/ usr/share/giella/mobilespellers/crk-mobile.zhfst crk.zhfst
```