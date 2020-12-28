import boto3
import os
import subprocess
from urllib.parse import unquote_plus

s3_client = boto3.client('s3')


def get_video_duration(video_path: str):
    print('Calling ffprobe to get video length in seconds')
    result = subprocess.run(['ffprobe', '-v', 'error', '-select_streams', 'v:0', '-show_entries',
                             'stream=duration', '-of', 'default=noprint_wrappers=1:nokey=1', video_path], stdout=subprocess.PIPE)

    print(f'Call result: {result}')
    duration = float(result.stdout.decode('utf-8').strip())
    print(f'Video duration: {duration}')
    return duration


def generate_frames(video_path: str, frame_prefix: str, n: int, start: float, width: int, height: int):
    print(f'Calling ffmpeg to seek and extract {n} frames')
    result = subprocess.call(['ffmpeg', '-ss', str(start), '-i',
                              video_path, '-frames:v', str(n), '-s', f'{width}x{height}', '-y', f'{frame_prefix}%03d.png'])
    print(f'Call result: {result}')


def build_preview(frame_prefix: str, preview_path: str):
    print(f'Calling ffmpeg to generate preview')
    result = subprocess.call(
        ['ffmpeg', '-i', f'{frame_prefix}%03d.png', '-y', preview_path])
    print(f'Call result: {result}')


def lambda_handler(event, context):
    for record in event['Records']:
        bucket = record['s3']['bucket']['name']
        video_key = unquote_plus(record['s3']['object']['key'])
        video_id = video_key.split('.')[0]

        video_path = f'/tmp/{video_key}'
        s3_client.download_file(bucket, video_key, video_path)

        if not os.path.isfile(video_path):
            print(f'{video_path} doesn\'t exist or isn\'t a file!')
            raise Exception(
                f'Couldn\'t download {video_key} from bucket {bucket}')

        frame_prefix = '/tmp/frame'
        preview_path = f'/tmp/{video_id}.gif'

        number_of_frames = 30
        start = get_video_duration(video_path) / 2
        generate_frames(video_path, frame_prefix,
                        number_of_frames, start, 240, 135)
        build_preview(frame_prefix, preview_path)

        if not os.path.isfile(preview_path):
            print(f'{preview_path} doesn\'t exist or isn\'t a file!')
            raise Exception(f'Couldn\'t generate preview from video')

        s3_client.upload_file(preview_path, 'minitube.previews',
                              f'{video_id}.gif', ExtraArgs={'ACL': 'public-read'})
