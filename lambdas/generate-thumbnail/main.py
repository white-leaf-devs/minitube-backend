import cv2
from PIL import Image

def getThumbnails(video, total_frames, jumps=5):
    thumbnails = []

    selected_frames = [i*(total_frames//jumps) for i in range(jumps)]

    success = True
    counter = 0
    while success:
        success, pixels = video.read()

        if counter in selected_frames:
            img = Image.fromarray(pixels.astype('uint8'), 'RGB').resize((240,135))
            thumbnails.append(img)

        counter += 1

    return thumbnails


def buildThumbnails(in_filename, out_file_prefix):
    video = cv2.VideoCapture(in_filename)

    frame_count = int(video.get(cv2.CAP_PROP_FRAME_COUNT))
    thumbnails = getThumbnails(video, frame_count)

if __name__ == '__main__':
    buildThumbnails('La dura vida de Rubius.mp4', 'nani')