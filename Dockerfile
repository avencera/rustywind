FROM alpine:3.14

RUN apk --no-cache add curl --virtual .build-deps && \
    curl -LSfs https://avencera.github.io/rustywind/install.sh | sh -s -- --git avencera/rustywind && \
    apk del .build-deps

ENTRYPOINT [ "rustywind" ]
