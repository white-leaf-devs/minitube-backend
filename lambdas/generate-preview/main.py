import boto3
import json
import cv2
import os
import sys
import uuid
from urllib.parse import unquote_plus
from PIL import Image

s3_client = boto3.client('s3')


def get_frames(video, start_frame, number=30):
    frames = []

    success = True
    counter = 0
    while success and counter < start_frame+number:
        success, pixels = video.read()

        if counter >= start_frame and success:
            img = cv2.resize(pixels, (240, 135))
            frames.append(img)

        counter += 1

    return frames


def build_preview(in_filename, out_filename, out_file_prefix):
    video = cv2.VideoCapture(in_filename)

    frame_count = int(video.get(cv2.CAP_PROP_FRAME_COUNT))
    resized_frames = get_frames(video, frame_count/2)

    frames = []

    for i, frame in enumerate(resized_frames):
        filename = f'{out_file_prefix}_{i}.png'
        cv2.imwrite(filename, frame)

        im = Image.open(filename)
        frames.append(im)

    frames[0].save(out_filename, save_all=True,
                   append_images=frames[1:], optimize=True)


def lambda_handler(event, context):
    for record in event['Records']:
        bucket = record['s3']['bucket']['name']
        key = unquote_plus(record['s3']['object']['key'])
        tmpkey = key.replace('/', '')

        tmpkey_no_extension = os.path.splitext(tmpkey)[0]

        file_id = uuid.uuid4()

        download_path = '/tmp/{}{}'.format(file_id, tmpkey)
        print(download_path)
        upload_path = '/tmp/preview-{}.gif'.format(tmpkey_no_extension)
        print(upload_path)
        s3_client.download_file(bucket, key, download_path)

        prefix = f'/tmp/{tmpkey_no_extension}_frame'

        build_preview(download_path, upload_path, prefix)
        s3_client.upload_file(upload_path, 'minitube.previews',
                              f'{tmpkey_no_extension}.gif', ExtraArgs={'ACL': 'public-read'})
