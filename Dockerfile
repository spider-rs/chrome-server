FROM zenika/alpine-chrome

EXPOSE 9222

USER root

RUN apk add --no-cache tini

USER chrome

ENTRYPOINT ["tini", "--"]