import boto3
import json
import cv2
import os
import sys
import uuid
from urllib.parse import unquote_plus
import base64

s3_client = boto3.client('s3')



def getThumbnails(video, total_frames, jumps=5):
    thumbnails = []

    selected_frames = [i*(total_frames//jumps) for i in range(jumps)]

    success = True
    counter = 0
    while success:
        success, pixels = video.read()

        if counter in selected_frames:
            img = cv2.resize(pixels, (240,135))
            thumbnails.append(img)
        
        counter += 1

    return thumbnails


def buildThumbnails(in_filename, out_file_prefix):
    video = cv2.VideoCapture(in_filename)

    frame_count = int(video.get(cv2.CAP_PROP_FRAME_COUNT))
    thumbnails = getThumbnails(video, frame_count)

    thumbnails_as_base64 = []

    for i,thum in enumerate(thumbnails):
        filename = f'{out_file_prefix}_{i}.png'
        cv2.imwrite(filename, thum)

        with open(filename, 'rb') as image_file:
            encoded_string = str(base64.b64encode(image_file.read()))
            thumbnails_as_base64.append(encoded_string)

    return thumbnails_as_base64

def lambda_handler(event, context):
    for record in event['Records']:
        bucket = record['s3']['bucket']['name']
        key = unquote_plus(record['s3']['object']['key'])
        tmpkey = key.replace('/', '')
        
        tmpkey_no_extension = os.path.splitext(tmpkey)[0]
        
        file_id = uuid.uuid4()
        
        download_path = '/tmp/{}{}'.format(file_id, tmpkey)
        print(download_path)
        s3_client.download_file(bucket, key, download_path)
        
        upload_path_prefix = '/tmp/thumb-{}'.format(tmpkey_no_extension)
        print(upload_path_prefix)
        
        thumbnails = buildThumbnails(download_path, upload_path_prefix)
        
        return {
            'statusCode': 200,
            'body': json.dumps(thumbnails)
        }