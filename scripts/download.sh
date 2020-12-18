#!/bin/bash

if [[ ! -f "${2}" ]]; then 
    echo "Dowloading from ${1}"
    wget "${1}" -O "${2}"
fi 