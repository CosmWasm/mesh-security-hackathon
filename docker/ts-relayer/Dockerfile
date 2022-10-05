FROM node:18-alpine3.16

# Set up dependencies
ENV PACKAGES curl make bash jq

# Install minimum necessary dependencies
RUN apk add --no-cache $PACKAGES

# Install the relayer
RUN npm i -g @confio/relayer@main

WORKDIR /opt
