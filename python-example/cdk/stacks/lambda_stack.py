from aws_cdk import (
    Stack,
    aws_lambda as _lambda,
    aws_iam as iam,
    aws_logs as logs,
    Duration,
    CfnOutput
)
from constructs import Construct

class LambdaObservabilityStack(Stack):
    def __init__(self, scope: Construct, construct_id: str, **kwargs) -> None:
        super().__init__(scope, construct_id, **kwargs)

        # Lambda execution role with Bedrock and X-Ray permissions
        lambda_role = iam.Role(
            self, "LambdaRole",
            assumed_by=iam.ServicePrincipal("lambda.amazonaws.com"),
            managed_policies=[
                iam.ManagedPolicy.from_aws_managed_policy_name("service-role/AWSLambdaBasicExecutionRole"),
                iam.ManagedPolicy.from_aws_managed_policy_name("AWSXRayDaemonWriteAccess")
            ],
            inline_policies={
                "BedrockAccess": iam.PolicyDocument(
                    statements=[
                        iam.PolicyStatement(
                            effect=iam.Effect.ALLOW,
                            actions=["bedrock:InvokeModel"],
                            resources=["*"]
                        )
                    ]
                )
            }
        )

        # Log group
        log_group = logs.LogGroup(
            self, "LambdaLogGroup",
            retention=logs.RetentionDays.ONE_WEEK
        )

        # Lambda function
        lambda_function = _lambda.DockerImageFunction(
            self, "LangChainFunction",
            code=_lambda.DockerImageCode.from_image_asset("../lambda-function"),
            role=lambda_role,
            timeout=Duration.minutes(5),
            memory_size=1024,
            tracing=_lambda.Tracing.ACTIVE,
            environment={
                "AWS_LAMBDA_EXEC_WRAPPER": "/opt/otel-instrument",
                "OTEL_PROPAGATORS": "tracecontext,baggage,xray",
                "OTEL_PYTHON_DISABLED_INSTRUMENTATIONS": "urllib3"
            },
            log_group=log_group
        )

        # Function URL for easy testing
        function_url = lambda_function.add_function_url(
            auth_type=_lambda.FunctionUrlAuthType.NONE,
            cors=_lambda.FunctionUrlCorsOptions(
                allowed_origins=["*"],
                allowed_methods=[_lambda.HttpMethod.POST],
                allowed_headers=["*"]
            )
        )

        CfnOutput(self, "FunctionUrl", value=function_url.url)
        CfnOutput(self, "FunctionName", value=lambda_function.function_name)
