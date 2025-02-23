#!/bin/bash

SRC=$(realpath $(cd -P "$(dirname "${BASH_SOURCE[0]}")" && pwd))

OUT=$SRC/out
TARGETS=()
TAGS=()
VERSION=
IMAGE=docker.io/spiderrust/headless-shell

OPTIND=1
while getopts "o:t:g:v:i:" opt; do
case "$opt" in
  o) OUT=$OPTARG ;;
  t) TARGETS+=($OPTARG) ;;
  g) TAGS+=($OPTARG) ;;
  v) VERSION=$OPTARG ;;
  i) IMAGE=$OPTARG ;;
esac
done

set -e

# check out dir
if [ ! -d $OUT ]; then
  echo "$OUT does not exist!"
  exit 1
fi

# determine version
if [ -z "$VERSION" ]; then
  VERSION=$(ls $OUT/*.bz2 | sort -r -V | head -1 | sed -e 's/.*headless-shell-\([0-9\.]\+\).*/\1/')
fi

# determine targets
if [ ${#TARGETS[@]} -eq 0 ]; then
  TARGETS=($(ls $OUT/*-${VERSION}-*.bz2 | sed -e 's/.*headless-shell-[0-9\.]\+-\([a-z0-9]\+\).*/\1/' | xargs))
fi

# join_by ',' ${A[@]} ${B[@]}
join_by() {
  local d=${1-} f=${2-}
  if shift 2; then
    printf %s "$f" "${@/#/$d}"
  fi
}

echo "VERSION:  $VERSION [${TARGETS[@]}]"
echo "IMAGE:    $IMAGE [tags: $(join_by ' ' $VERSION ${TAGS[@]})]"

IMAGES=()
for TARGET in ${TARGETS[@]}; do
  TAG=$VERSION-$TARGET
  NAME=$IMAGE:$TAG
  IMAGES+=($NAME)

  if [ -n "$(docker images -q $NAME)" ]; then
    echo -e "\n\nSKIPPING BUILD FOR $NAME ($(date))"
    continue
  fi

  echo -e "\n\nBUILDING $NAME ($(date))"
  ARCHIVE=$OUT/headless-shell-$VERSION-$TARGET.tar.bz2
  if [ ! -f $ARCHIVE ]; then
    echo "ERROR: $ARCHIVE is missing!"
    exit 1
  fi

  # Make sure to extract the archive if necessary and adjust context as needed
  rm -rf $OUT/$VERSION-$TARGET
  mkdir -p $OUT/$VERSION-$TARGET
  tar -C $OUT/$VERSION-$TARGET -jxf $ARCHIVE

  (set -x;
    docker build \
      --platform linux/$TARGET \
      --build-arg VERSION="$VERSION-$TARGET" \
      --tag $NAME \
      $OUT/$VERSION-$TARGET
  )
done