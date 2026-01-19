#!/usr/bin/env bash

set -x

exit_handler() {
    exit "${TEST_EXIT_STATUS:-1}"
}
trap exit_handler TERM
trap '
  pids=$(jobs -p)
  if [ -n "$pids" ]; then
    kill $pids
  fi
  exit $TEST_EXIT_STATUS
' INT EXIT

export RUST_BACKTRACE=1
export RUST_LOG=debug

NC="\e[39m"
GREEN="\e[32m"
RED="\e[31m"

function run_test {
  DIR=$(mktemp -d)
  LOG_FILE=$(mktemp)
  cargo build --example hello > /dev/null 2>&1
  echo "Starting hello example with mount point: $DIR"
  cargo run --example hello -- $DIR $2 > "$LOG_FILE" 2>&1 &
  FUSE_PID=$!
  sleep 10

  echo "mounting at $DIR"
  echo "Hello example output:"
  cat "$LOG_FILE"
  echo "---"
  echo "Directory contents:"
  ls -la "$DIR" || echo "Failed to list directory"
  echo "---"

  if [[ $(cat ${DIR}/hello.txt) = "Hello World!" ]]; then
      echo -e "$GREEN OK $1 $2 $NC"
  else
      echo -e "$RED FAILED $1 $2 $NC"
      export TEST_EXIT_STATUS=1
      exit 1
  fi

  kill $FUSE_PID
  wait $FUSE_PID
}

run_test 'with libfuse'

# TODO: re-enable this test. It seems to hang on OSX
#run_test --features=libfuse 'with libfuse' --auto_unmount

export TEST_EXIT_STATUS=0
