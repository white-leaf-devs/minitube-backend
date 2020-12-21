import boto3
import json
import os
import time
import base64
import subprocess

s3_client = boto3.client('s3')

def genThumbnail(video_path, thumbnail_path, timestamp):
    result = subprocess.call(('ffmpeg','-i', video_path, '-ss', timestamp, '-vframes', '1', thumbnail_path))
    print('RESULT OF FFMPEG CALL')
    print(result)


def lambda_handler(event, context):
    bucket = 'minitube.video'
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

    genThumbnail(download_path, upload_path, timestamp)

    s3_client.upload_file(upload_path, 'minitube.thumbnail', f'{tmpkey_no_extension}.png', ExtraArgs={'ACL': 'public-read'})
