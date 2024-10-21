#!/bin/sh

FILE="$(pwd)/pictures/1.jpeg"

curl -F "file=@\"${FILE}\"" "http://127.0.0.1:8080/cat"
