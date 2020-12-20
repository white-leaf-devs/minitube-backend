FROM amazon/aws-lambda-python:3.8
RUN yum install -y wget tar xz openssl11-libs \
    && pip install Pillow boto3 awslambdaric \
    && wget https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-amd64-static.tar.xz -O ffmpeg.tar.xz \
    && tar -xf ffmpeg.tar.xz \
    && rm ffmpeg.tar.xz \
    && mv ffmpeg*/* /usr/bin 
COPY main.py ${LAMBDA_TASK_ROOT}
CMD [ "main.lambda_handler" ]