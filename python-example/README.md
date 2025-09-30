# Tracing Python-based containerized Lambda with X-Ray and OpenTelemetry

In this example, we show how to install and use the AWS Lambda OpenTelemetry instrumentation layer for Python inside a container-based function (since normally, layers can only be added to runtime-based functions). The function itself calls a foundation model on Amazon Bedrock, via LangChain.


## Prerequisites

- AWS CLI configured with appropriate permissions to your target AWS Account
- Docker (or a compatible alternative) installed and running
- Python 3.13+ installed
- Node.js 18+ (for CDK CLI)


## Setup

1. Install CDK CLI:
```bash
npm install -g aws-cdk
```

2. Create Python virtual environment and install dependencies:
```bash
cd cdk
python3 -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
```

3. In the ["Model access" page](https://console.aws.amazon.com/bedrock/home?#/modelaccess) of the Amazon Bedrock console in your target AWS account and region, enable access to the Foundation Model you'll use for testing the example. The default model ID is configured via environment variable fallback in [lambda-function/index.py](lambda-function/index.py). You can find IDs for other models in the [Bedrock model catalog](https://console.aws.amazon.com/bedrock/home?#/model-catalog) or [Cross-region inference catalog](https://console.aws.amazon.com/bedrock/home?#/inference-profiles).

4. Bootstrap CDK (first time only):
```bash
cdk bootstrap
```


## (Optional) Testing the container locally

If you want to test the containerized function locally first, you can build and start the container as follows:

```bash
cd lambda-function
docker build -t langchain-lambda .
docker run --rm -p 9000:8080 langchain-lambda
```

...and then test it:

```bash
curl -X POST "http://localhost:9000/2015-03-31/functions/function/invocations" \
  -d '{"inputText": "Hello world"}'
```


## Deploy

CDK will automatically build the Docker image and deploy:

```bash
cdk deploy
```

This process will:
1. Build the Docker image from `../lambda/Dockerfile` (Python 3.13 runtime)
2. Push the image to ECR
3. Deploy the Lambda function using the container image
4. Set up X-Ray tracing and OpenTelemetry instrumentation


## Test the Function

After deployment, you can invoke the function through the test UI in the [AWS Lambda Console](https://console.aws.amazon.com/lambda/home?#), or from your local terminal using the AWS CLI:

```bash
aws lambda invoke \
  --function-name <FunctionName> \
  --payload '{"inputText": "What is AWS Lambda?"}' \
  response.json && cat response.json
```


## Monitor Observability

1. **X-Ray Traces**: Go to [Traces](https://console.aws.amazon.com/cloudwatch/home?#xray:traces/query) under CloudWatch Application Signals, or the AWS X-Ray console to view distributed traces
2. **CloudWatch Logs**: Check `/aws/lambda/LangChainLambdaStack-LangChainFunction*` log group
3. **CloudWatch Metrics**: View Lambda metrics and custom OpenTelemetry metrics


## Clean Up

```bash
cdk destroy
```


## Key Observability Features

- **X-Ray Tracing**: Automatic distributed tracing enabled
- **OpenTelemetry**: LangChain operations instrumented with AWS OTEL layer
- **CloudWatch Integration**: Logs and metrics collection
- **Custom Spans**: Bedrock API calls traced automatically
