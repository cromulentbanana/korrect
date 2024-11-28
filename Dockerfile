FROM rust:latest
LABEL maintainer dan@badpacket.in

WORKDIR /project
COPY . /project

# Make build script executable
RUN chmod +x build.sh

# Set environment to indicate Docker build
ENV DOCKER_BUILD=1

RUN ./build.sh

FROM alpine:latest
COPY --from=0 /project/dist/* /usr/local/bin/
ENTRYPOINT ["/usr/local/bin/korrect-x86_64-linux"]
