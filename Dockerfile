# Ubuntu latest
FROM ubuntu:latest
# Update packages
RUN apt-get update && apt-get upgrade -y
RUN apt install curl -y
# use the mounted ordinals directory from the host
WORKDIR /ordinals
# Install latest ord release from github 
RUN curl -Ls -o install.sh https://raw.githubusercontent.com/ordinals/ord/master/install.sh
# Make executable
RUN chmod 755 install.sh
# Install to /ordinals inside docker container
RUN ./install.sh --to /ordinals
# Set entrypoint to ord with env vars set in docker-compose.yml
ENTRYPOINT /ordinals/ord --data-dir $DATA_DIR --rpc-url $RPC_HOST:$RPC_PORT --cookie-file $COOKIE_FILE server --http --http-port $HTTP_PORT
