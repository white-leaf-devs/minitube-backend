import boto3
import json
import os
import time
import base64
import subprocess
from PIL import Image

s3_client = boto3.client('s3')


def gen_thumbnail(video_path, thumbnail_path, timestamp):
    result = subprocess.call(
        ('ffmpeg', '-ss', timestamp, '-i', video_path, '-vframes', '1', thumbnail_path))
    print('RESULT OF FFMPEG CALL')
    print(result)


def resize_thumbnail(thumbnail_path, width, height):
    im = Image.open(thumbnail_path)
    im = im.resize((width, height))
    im.save(thumbnail_path)


def lambda_handler(event, context):
    bucket = 'minitube.videos'
    key = event['video_key']
    timestamp_in_seconds = int(event['timestamp'])
    timestamp = time.strftime('%H:%M:%S', time.gmtime(timestamp_in_seconds))

    tmpkey = key.replace('/', '')
    tmpkey_no_extension = os.path.splitext(tmpkey)[0]

    download_path = '/tmp/{}'.format(tmpkey)
    print(download_path)
    s3_client.download_file(bucket, key, download_path)

    upload_path = '/tmp/thumb_{}.png'.format(tmpkey_no_extension)
    print(upload_path)

    gen_thumbnail(download_path, upload_path, timestamp)
    resize_thumbnail(upload_path, 240, 135)

    s3_client.upload_file(upload_path, 'minitube.thumbnails',
                          f'{tmpkey_no_extension}.png', ExtraArgs={'ACL': 'public-read'})
