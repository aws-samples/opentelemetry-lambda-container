mod observability;
use aws_config;
use aws_lambda_events::event::s3::S3Event;
use aws_sdk_rekognition as rekognition;
use aws_sdk_rekognition::operation::detect_labels::{DetectLabelsError, DetectLabelsOutput};
use lambda_runtime::{service_fn, Error, LambdaEvent};
use observability::{get_span_context_from_environment_var, get_trace_id, init_observability};

use opentelemetry::global::{self,shutdown_tracer_provider};
use opentelemetry::{KeyValue,Context};
use opentelemetry::trace::{Span, Tracer,FutureExt, TraceContextExt};
use serde_json::{json, Value};
use simple_error::SimpleError;
use tracing::{debug,info};

#[allow(unused_imports)]
use mockall::automock;

trait Rekognition {
    fn new(inner: rekognition::Client) -> Self;
    async fn detect_labels(
        &self,
        image: rekognition::types::Image,
    ) -> Result<DetectLabelsOutput, rekognition::error::SdkError<DetectLabelsError>>;
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct RekognitionImpl {
    inner: rekognition::Client,
}

impl Rekognition for RekognitionImpl {
    #[allow(dead_code)]
    fn new(inner: rekognition::Client) -> Self {
        Self { inner }
    }

    #[allow(dead_code)]
    async fn detect_labels(
        &self,
        image: rekognition::types::Image,
    ) -> Result<DetectLabelsOutput, rekognition::error::SdkError<DetectLabelsError>> {
        self.inner.detect_labels().image(image).send().await
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    init_observability()?;
    let func = service_fn(handler);
    lambda_runtime::run(func).await?;
    Ok(())
}

#[derive(Debug, PartialEq)]
struct DetectLabelArguments {
    bucket: String,
    name: String,
}

fn retrieve_arguments_from_event(event: S3Event) -> Result<DetectLabelArguments, String> {
    let record = &event.records[0];
    let bucket = record.s3.bucket.name.as_ref().ok_or("No bucket name")?;
    let object_name = record.s3.object.key.as_ref().ok_or("No object name")?;
    Ok(DetectLabelArguments {
        bucket: bucket.to_string(),
        name: object_name.to_string(),
    })
}

async fn detect_labels<T: Rekognition + std::fmt::Debug>(
    client: T,
    arg: DetectLabelArguments,
) -> Result<Vec<String>, rekognition::Error> {
    let trace_id = get_trace_id();
    info!("Trace ID(in detect_labels) is {trace_id}");
    let image = construct_rekognition_image(&arg);
    let tracer = global::tracer("lambda-tracer");
    let mut span = tracer.start("detect-label");
    let output = client.detect_labels(image).await?;

    match output.labels {
        Some(labels) => {
            span.set_attribute(KeyValue::new("label_num", labels.len() as i64));
            Ok(labels
                .iter()
                .map(|label| label.clone().name.unwrap())
                .collect::<Vec<String>>())
        }
        None => {
            Context::current()
                .span()
                .set_attribute(KeyValue::new("label_num", 0));
            Ok(vec![])
        }
    }
}

fn construct_rekognition_image(arg: &DetectLabelArguments) -> rekognition::types::Image {
    rekognition::types::Image::builder()
        .set_s3_object(Some(
            rekognition::types::S3Object::builder()
                .set_bucket(Some(arg.bucket.to_string()))
                .set_name(Some(arg.name.to_string()))
                .build(),
        ))
        .build()
}



async fn handler(event: LambdaEvent<S3Event>) -> Result<Value, Error> {
    let parent_context = get_span_context_from_environment_var()?;
    debug!("Event is {:?}", &event);
    debug!("Parent Context is {:?}", &parent_context);
    let config = aws_config::load_from_env().await;
    let rekognition_client = rekognition::Client::new(&config);
    let client = RekognitionImpl::new(rekognition_client);
    let (event, _context) = event.into_parts();
    debug!("S3Event is {:?}", &event);
    debug!("Context is {:?}", &_context);
    let argument = retrieve_arguments_from_event(event)
        .map_err(|e| Box::new(SimpleError::new(format!("Invalid event: {e}"))))?;

    let labels = detect_labels(client, argument)
        .with_context(Context::new().with_remote_span_context(parent_context))
        .await?;
    info!("Labels is {:?}", &labels);
    shutdown_tracer_provider();
    Ok(json!({ "message": format!("Labels is {:?}!", labels) }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake_opentelemetry_collector::{setup_tracer, FakeCollectorServer};
    use mockall::predicate::*;
    use mockall::*;

    use opentelemetry::trace::TraceId;

    use std::env::set_var;
    use std::fs::File;
    use std::io::BufReader;
    #[cfg(test)]
    mock! {
        #[derive(Debug)]
        pub RekognitionImpl {}
        impl Rekognition for RekognitionImpl {
            fn new(inner: rekognition::Client) -> Self;
            async fn detect_labels(
                &self,
                image: rekognition::types::Image,
            ) -> Result<DetectLabelsOutput, rekognition::error::SdkError<DetectLabelsError>>;
        }
    }

    fn setup_test_event() -> Result<S3Event, Error> {
        let key = "_X_AMZN_TRACE_ID";
        set_var(key,"Root=1-65dc5008-1561ed7046ffcbcb114af027;Parent=b510129166d5a083;Sampled=1;Lineage=f98dd9ff:0");
        let file = File::open("events/s3.json")?;
        let reader = BufReader::new(file);
        let event: S3Event = serde_json::from_reader(reader)?;
        Ok(event)
    }

    #[tokio::test]
    async fn test_retrieve_arguments_from_event() -> Result<(), Error> {
        let event = setup_test_event()?;
        let result = retrieve_arguments_from_event(event);
        let expected = DetectLabelArguments {
            bucket: "DOC-EXAMPLE-BUCKET".to_string(),
            name: "b21b84d653bb07b05b1e6b33684dc11b".to_string(),
        };
        match result {
            Ok(actual) => assert_eq!(actual, expected),
            Err(e) => panic!("Failed: {:?}", e),
        }
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_detect_labels() {
        let event = setup_test_event().unwrap();
        let fake_collector = FakeCollectorServer::start()
            .await
            .expect("fake collector setup and started");
        let _tracer = setup_tracer(&fake_collector).await;
        let parent_context = get_span_context_from_environment_var().unwrap();
        let argument = retrieve_arguments_from_event(event).unwrap();
        let expected_label = "landscape";
        let _image = construct_rekognition_image(&argument);
        let expected_detect_labels_output = DetectLabelsOutput::builder()
            .set_labels(Some(vec![rekognition::types::Label::builder()
                .set_name(Some(expected_label.to_string()))
                .build()]))
            .build();
        let mut mock_client = MockRekognitionImpl::default();
        mock_client
            .expect_detect_labels()
            .return_once(|_image| Ok(expected_detect_labels_output));
        let actual = detect_labels(mock_client, argument)
            .with_context(Context::new().with_remote_span_context(parent_context))
            .await;
        shutdown_tracer_provider();

        let otel_spans = fake_collector.exported_spans();
        assert!(actual.is_ok());
        assert_eq!(actual.unwrap(), vec![expected_label.to_string()]);
        let parent_context_for_assert = get_span_context_from_environment_var().unwrap();
        assert_eq!(
            otel_spans
                .clone()
                .into_iter()
                .map(|span| span.parent_span_id)
                .collect::<Vec<String>>(),
            vec![parent_context_for_assert.span_id().to_string()]
        );
    }
    #[test]
    fn test_convert_traceid_xray_to_otel() {
        let _ = setup_test_event().unwrap();
        let span_context = get_span_context_from_environment_var();
        assert!(span_context.is_ok());
        assert_eq!(
            span_context.clone().unwrap().trace_id(),
            TraceId::from_hex("65dc50081561ed7046ffcbcb114af027").unwrap()
        );
        assert_eq!(
            span_context.clone().unwrap().span_id().to_string(),
            "b510129166d5a083".to_string()
        );
    }

    async fn async_func() -> TraceId {
        let trace_id = get_trace_id();
        trace_id
    }
    #[tokio::test]
    async fn test_trace_id_in_async_function_call() {
        init_observability().unwrap();
        let _ = setup_test_event().unwrap();
        let parent_context = get_span_context_from_environment_var().unwrap();

        let expected_trace_id = parent_context.trace_id();
        let actual_trace_id = async_func()
            .with_context(Context::new().with_remote_span_context(parent_context))
            .await;
        assert_eq!(expected_trace_id, actual_trace_id);
    }
}
