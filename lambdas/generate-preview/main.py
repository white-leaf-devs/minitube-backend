import cv2
from PIL import Image

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

if __name__ == '__main__':
    buildPreview('La dura vida de Rubius.mp4', 'test.gif')