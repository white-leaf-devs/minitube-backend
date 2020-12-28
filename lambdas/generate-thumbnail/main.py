import boto3
import os
import subprocess

s3_client = boto3.client('s3')


def generate_thumbnail(video_path: str, thumbnail_path: str, timestamp: float, width: int, height: int):
    print('Calling ffmpeg to seek and extract one frame')
    print(f'Timestamp (in seconds): {timestamp}')

    result = subprocess.call(
        ['ffmpeg', '-ss', str(timestamp), '-i', video_path, '-frames:v', '1', '-s', f'{width}x{height}', '-y', thumbnail_path], timeout=None)
    print(f'Call result: {result}')


def lambda_handler(event, context):
    os.chdir('/tmp')

    bucket = event['bucket']
    video_key = event['video_key']
    timestamp = float(event['timestamp'])
    video_id = video_key.split('.')[0]

    video_path = f'{video_key}'
    s3_client.download_file(bucket, video_key, video_path)

    if not os.path.isfile(video_path):
        print(f'{video_path} doesn\'t exist or isn\'t a file!')
        raise Exception(f'Couldn\'t download {video_key} from bucket {bucket}')

    thumbnail_path = f'{video_id}.png'
    generate_thumbnail(video_path, thumbnail_path, timestamp, 240, 135)

    if not os.path.isfile(thumbnail_path):
        print(f'{thumbnail_path} doesn\'t exist or isn\'t a file!')
        raise Exception(f'Couldn\'t generate thumbnail from video')

    s3_client.upload_file(thumbnail_path, 'minitube.thumbnails',
                          f'{video_id}.png', ExtraArgs={'ACL': 'public-read'})
