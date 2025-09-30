#!/usr/bin/env node
import 'source-map-support/register';
import * as cdk from 'aws-cdk-lib';
import { LambdaTraceStack } from '../lib/lambda-trace-stack';

const app = new cdk.App();
new LambdaTraceStack(app, 'LambdaTraceStack', {});