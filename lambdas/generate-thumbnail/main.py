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

def generate_n_thumbnails(in_filename, out_file_prefix, n=5):
  vid = ffmpeg.probe(in_filename)
  duration = float(vid['streams'][0]['duration'])

  for i in range(n):
    generate_thumbnail(in_filename, f'{out_file_prefix}_{i+1}.png', (i/n)*duration, 120)


if __name__ == '__main__':
    generate_n_thumbnails('La dura vida de Rubius.mp4', 'nani')