FROM public.ecr.aws/lambda/python:3.13-x86_64

ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1

# Download the OpenTelemetry Layer with AppSignals Support:
RUN dnf install -y unzip wget
RUN wget https://github.com/aws-observability/aws-otel-python-instrumentation/releases/latest/download/layer.zip -O /tmp/layer.zip

# Extract and include Lambda layer contents:
RUN mkdir -p /opt && \
    unzip /tmp/layer.zip -d /opt/ && \
    chmod -R 755 /opt/ && \
    rm /tmp/layer.zip

# Copy just requirements.txt file into the container first:
COPY requirements.txt ./
# Install dependencies found in your requirements.txt file:
RUN pip install --no-cache-dir -r requirements.txt

# Copy over the rest of the code (so the above install steps don't get rebuilt if only the Python
# source has changed):
COPY . .

# Point to the handler function of your lambda function:
CMD ["index.handler"]
