ROOT=$PWD
DIST=$ROOT/dist
rm -rf $DIST && mkdir $DIST

echo "building msoffice addin"
git clone https://github.com/divvun/divvun-gramcheck-web.git || (cd divvun-gramcheck-web && git pull && cd ..)
cd divvun-gramcheck-web/msoffice
npm ci
npm run build
mkdir $DIST/msoffice
cp -R dist/* $DIST/msoffice/

cd $ROOT

echo "copying configs"
cp docker-compose.yml $DIST
cp Caddyfile $DIST
cp run.sh $DIST
cp divvun-api.service $DIST

docker build -t divvun/divvun-api ..
docker save divvun/divvun-api | gzip > $DIST/divvun-api.tar.gz

tar cvvfz dist.tar.gz dist
