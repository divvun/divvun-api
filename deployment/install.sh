gunzip -c divvun-api.tar.gz | docker image load
mkdir -p data/grammar
mkdir -p data/spelling
mkdir -p data/hyphenation
cp divvun-api.service /etc/systemd/system/
