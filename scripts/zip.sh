#!/bin/bash

dir=$(pwd)
cd "${1}"

zip "${dir}/${2}" *

