#!/usr/bin/env python3
import aws_cdk as cdk
from stacks.lambda_stack import LambdaObservabilityStack

app = cdk.App()
LambdaObservabilityStack(app, "LangChainLambdaStack")
app.synth()
