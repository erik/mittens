#!/bin/bash

#written by zzmp

# This script will recompile a rust project using `make`
# every time something in the specified directory changes.

# Watch files in infinite loop
watch () {
  UNAME=$(uname)
  if [ -e "$2" ]; then
    echo "Watching files in $2.."
    CTIME=$(date "+%s")
    while :; do
      sleep 1
      for f in `find $2 -type f -name "*.rs"`; do
        if [[ $UNAME == "Darwin" ]]; then
          st_mtime=$(stat -f "%m" "$f")
        elif [[ $UNAME == "FreeBSD" ]]; then
          st_mtime=$(stat -f "%m" "$f")
        else
          st_mtime=$(stat -c "%Y" "$f")
        fi
        if [ $st_mtime -gt $CTIME ]; then
          CTIME=$(date "+%s")
          echo "~~~ Rebuilding"
          $1
          if [ ! $? -eq 0 ]; then
            echo ""
          fi
        fi
      done
    done
  else
    echo "$2 is not a valid directory"
  fi
}

# Capture user input with defaults
CMD=${1:-make}
DIR=${2:-src}

if [ ${CMD:0:2} = '-h' ]; then
echo '
This script will recompile a rust project using `make`
every time something in the specified directory changes.

Use: ./watch.sh [CMD] [DIR]
Example: ./watch.sh "make run" src

CMD: Command to execute
     Complex commands may be passed as strings
     `make` by default
DIR: Directory to watch
     src by default

If DIR is supplied, CMD must be as well.
'
else
  watch "$CMD" "$DIR"
fi

