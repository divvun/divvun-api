FROM ubuntu:18.04
WORKDIR /app
COPY . /app
RUN apt-get update && apt-get install -y wget && wget https://apertium.projectjj.com/apt/install-nightly.sh && bash install-nightly.sh && apt-get install -y divvun-gramcheck
ENTRYPOINT ["divvun-checker", "-a"]
