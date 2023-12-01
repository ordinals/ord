# Ubuntu latest
FROM ubuntu:latest
# Update packages
RUN apt-get update && apt-get upgrade -y
RUN apt install curl -y
WORKDIR ~/bin
# Install latest ord release from github
RUN curl -Ls -o install.sh https://raw.githubusercontent.com/ordinals/ord/master/install.sh
# Make executable
RUN chmod +x install.sh
RUN ./install.sh
# Set entrypoint to ord with env vars set in docker-compose.yml
ENTRYPOINT ~/bin/ord --data-dir $DATA_DIR --rpc-url $RPC_HOST:$RPC_PORT --cookie-file $COOKIE_FILE server --http --http-port $HTTP_PORT