#!/usr/bin/env bash

i=0;
while true; do
  if [[ i%2 -eq 1 ]]; then
    target/debug/sockrs -c
  elif [[ i%10 -eq 0 ]]; then
    target/debug/sockrs -p $((i%100))
  else
    target/debug/sockrs -o
  fi
  printf "%d\n" $i;
  ((i++))
  date
  sleep 1
done
