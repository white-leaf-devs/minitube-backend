import boto3
import json
import os
import glob
import time
import base64
import subprocess
from PIL import Image
from urllib.parse import unquote_plus

s3_client = boto3.client('s3')

def resizeFrame(frame_path, width, height):
    im = Image.open(frame_path)
    im = im.resize((width, height))
    return im

def get_video_duration(video_path):
    result = subprocess.run(['ffprobe', '-v', 'error', '-select_streams', 'v:0', '-show_entries', 'stream=duration', '-of','default=noprint_wrappers=1:nokey=1', video_path], stdout=subprocess.PIPE)
    print('RESULT OF FFPROBE CALL')
    print(result)

    duration_in_seconds = int(float(result.stdout.decode('utf-8').strip())) // 2
    duration = time.strftime('%H:%M:%S', time.gmtime(duration_in_seconds))
    return duration

def get_n_frames(video_path, frame_prefix_path, n, start_timestamp):
    result = subprocess.call(('ffmpeg', '-ss' , start_timestamp, '-i', video_path, '-frames:v', str(n), frame_prefix_path + r'%03d.png'))
    print('RESULT OF FFMPEG CALL')
    print(result)
    
    list_of_files_result = subprocess.run(['ls', '/tmp'], stdout=subprocess.PIPE)
    list_of_files = list_of_files_result.stdout.decode('utf-8').split('\n')

    frames = [f'/tmp/{f}' for f in list_of_files if 'png' in f]
    frames.sort()

    print('FRAMES')
    print(frames)
    return frames

def build_preview(frame_paths, preview_path):
    frames = []
    for filename in frame_paths:
        im = resizeFrame(filename, 240, 135)
        frames.append(im)
    
    frames[0].save(preview_path, save_all=True, append_images=frames[1:], optimize=True)


def lambda_handler(event, context):
    for record in event['Records']:
        bucket = record['s3']['bucket']['name']
        key = unquote_plus(record['s3']['object']['key'])

        tmpkey = key.replace('/', '')
        tmpkey_no_extension = os.path.splitext(tmpkey)[0]

        download_path = '/tmp/{}'.format(tmpkey)
        print(download_path)
        
        prefix_path = f'/tmp/frame'
        upload_path = '/tmp/preview_{}.gif'.format(tmpkey_no_extension)
        print(upload_path)

        s3_client.download_file(bucket, key, download_path)

        number_of_frames = 30
        start_timestamp = get_video_duration(download_path)
        frame_paths = get_n_frames(download_path, prefix_path, number_of_frames, start_timestamp)

        build_preview(frame_paths, upload_path)

        s3_client.upload_file(upload_path, 'minitube.preview', f'{tmpkey_no_extension}.gif', ExtraArgs={'ACL': 'public-read'})
