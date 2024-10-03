import { Stack, StackProps, CfnOutput, RemovalPolicy, Duration } from 'aws-cdk-lib';
import { Construct } from 'constructs';
import { aws_lambda as lambda, aws_s3 as s3, aws_lambda_event_sources as eventsources, aws_iam as iam, aws_logs as logs } from 'aws-cdk-lib';
import * as path from 'path';

export class LambdaTraceStack extends Stack {
  constructor(scope: Construct, id: string, props?: StackProps) {
    super(scope, id, props);

    const bucket = new s3.Bucket(this, 'RekognitionSourceBucket', {
      publicReadAccess: false,
      autoDeleteObjects: true,
      removalPolicy: RemovalPolicy.DESTROY,
      blockPublicAccess: s3.BlockPublicAccess.BLOCK_ALL,
    });

    const role = new iam.Role(this,"LambdaRole", {
      assumedBy: new iam.ServicePrincipal('lambda.amazonaws.com'),
      managedPolicies: [
        {
          managedPolicyArn: "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
        },
        {
          managedPolicyArn: "arn:aws:iam::aws:policy/AWSXrayWriteOnlyAccess"
        }
      ]
    });
    role.addToPolicy(new iam.PolicyStatement({
      actions: ["rekognition:DetectLabels"],
      resources: ["*"],
      effect: iam.Effect.ALLOW
    }));
    role.addToPolicy(new iam.PolicyStatement({
      actions: ["s3:GetObject"],
      resources: [`${bucket.bucketArn}`,`${bucket.bucketArn}/*`],
      effect: iam.Effect.ALLOW
    }));

    const lambdaFunction = new lambda.DockerImageFunction(this, 'Function', {
      code: lambda.DockerImageCode.fromImageAsset(path.join(__dirname, '../../lambda-function')),
      architecture: lambda.Architecture.X86_64,
      environment: {
        "RUST_LOG": "bootstrap=debug,error"
      },
      logRetention: logs.RetentionDays.FIVE_DAYS,
      role: role,
      tracing: lambda.Tracing.ACTIVE,
      timeout: Duration.minutes(1),
    });
    
    lambdaFunction.addEventSource(new eventsources.S3EventSource(bucket,{
      events: [ s3.EventType.OBJECT_CREATED ],
    }))

    new CfnOutput(this, "RekognitionSourceBucketName", {
      value: bucket.bucketName
    });
  }
}
