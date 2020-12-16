import ffmpeg

def generate_thumbnail(in_filename, out_filename, time, width):
  try:
    (
      ffmpeg
      .input(in_filename,ss=time)
      .filter('scale', width, -1)
      .output(out_filename, vframes=1)
      .run()
    )
  except ffmpeg.Error as e:
    print(e.stderr.decode(), file=sys.stderr)
    sys.exit(1)

if __name__ == '__main__':
    generate_thumbnail('La dura vida de Rubius.mp4', 'nani.png', 0.1, 120)