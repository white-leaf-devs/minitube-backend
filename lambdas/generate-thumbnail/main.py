import cv2
import base64

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
            encoded_string = base64.b64encode(image_file.read())
            thumbnails_as_base64.append(encoded_string)

    return thumbnails_as_base64

if __name__ == '__main__':
    buildThumbnails('La dura vida de Rubius.mp4', 'nani')