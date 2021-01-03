# Video Processing Pipeline

This repository contains all the code related to the backend of a simple 
video processing line:

1. Upload an MP4 video, this will generate its preview using `ffmpeg`.
2. Choose thumbnail, this will generate a thumbnail based on the given timestamp
using `ffmpeg`, after it's generated we store label information on DynamoDB.
3. Search videos using the previously generated labels.

## HTTP API 
The API specification is available on [`api/openapi.yml`](api/openapi.yml), a more human friendly documentation is available at the [repository page](https://white-leaf-devs.github.io/minitube-backend/).

## Terraform
Terraform configuration files are included, if you want to apply all the changes just run `make apply`.

**Note:** Elastic Container Registries must be manually created in AWS.


## Frontend 
The frontend code is available on: [MBlev/minitube-frontend](https://github.com/MBlev/minitube-frontend)

## Demo
[![Video demo](https://img.youtube.com/vi/PkbSKQ_fjEQ/0.jpg)](https://www.youtube.com/watch?v=PkbSKQ_fjEQ)

## License
This repository is licensed under the terms of the MIT License.

See [LICENSE](LICENSE) to see the full text.