# Tracing Containerized Lambda Functions with AWS X-Ray and OpenTelemetry SDK

This repository showcases how to send traces from [container-based AWS Lambda functions](https://docs.aws.amazon.com/lambda/latest/dg/images-create.html) to [AWS X-Ray](https://docs.aws.amazon.com/xray/latest/devguide/aws-xray.html) using the OpenTelemetry SDKs.

With the [planned end of support](https://docs.aws.amazon.com/xray/latest/devguide/xray-daemon-eos.html) for the X-Ray SDKs and daemon, OpenTelemetry is the recommended and industry-standard way to ingest trace data to AWS X-Ray / CloudWatch. While most of the [managed Lambda runtimes](https://docs.aws.amazon.com/lambda/latest/dg/lambda-runtimes.html) offer [layers](https://aws-otel.github.io/docs/getting-started/lambda) to help builders get started with OpenTelemetry collection, this sample offers a starting point for working with container-based functions.


## Get started

We offer two separate examples using slightly different approaches, both including infrastructure deployment code in [AWS CDK](https://docs.aws.amazon.com/cdk/v2/guide/home.html). Check out:

- [python-example](python-example/README.md) - instrumenting a Python-based function calling Generative AI models via LangChain
- [rust-example](rust-example/README.md) - instrumenting a Rust-based function analyzing images with Amazon Rekognition


## Security

See [CONTRIBUTING](CONTRIBUTING.md#security-issue-notifications) for more information.


## License

This library is licensed under the MIT-0 License. See the LICENSE file.
