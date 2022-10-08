FROM zenika/alpine-chrome

EXPOSE 9222

USER root

RUN apk add --no-cache tini curl

USER chrome

ENTRYPOINT ["tini", "--"]