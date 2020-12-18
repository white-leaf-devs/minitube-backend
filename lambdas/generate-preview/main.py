import boto3
import json
import cv2
import os
import sys
import uuid
from urllib.parse import unquote_plus
from PIL import Image

s3_client = boto3.client('s3')

def getFrames(video, start_frame, number=30):
    frames = []

    success = True
    counter = 0
    while success and counter<start_frame+number:
        success, pixels = video.read()

        if counter >= start_frame:
            img = Image.fromarray(pixels.astype('uint8'), 'RGB').resize((240,135))
            frames.append(img)
        
        counter += 1

    return frames
    
def buildPreview(in_filename, out_filename):
    video = cv2.VideoCapture(in_filename)

    frame_count = int(video.get(cv2.CAP_PROP_FRAME_COUNT))
    frames = getFrames(video, frame_count/2)

    frames[0].save(out_filename, save_all=True, append_images=frames[1:], optimize=True)

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
        
        buildPreview(download_path, upload_path)
        s3_client.upload_file(upload_path, 'minitube.preview', f'{tmpkey_no_extension}.gif')