use opentelemetry::trace::TracerProvider as _;
use opentelemetry::KeyValue;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_stdout as stdout;
use std::time::SystemTime;
use tracing::{error, span};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

fn main() {
    // Create a new OpenTelemetry trace pipeline that prints to stdout
    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(stdout::SpanExporter::default())
        .build();
    let tracer = provider.tracer("otel_span_event_example");

    // Create a tracing layer with the configured tracer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    // Use the tracing subscriber `Registry`, or any other subscriber
    // that impls `LookupSpan`
    let subscriber = Registry::default().with(telemetry);

    // Trace executed code
    tracing::subscriber::with_default(subscriber, || {
        // Spans will be sent to the configured OpenTelemetry exporter
        let root = span!(tracing::Level::INFO, "app_start", work_units = 2);
        let _enter = root.enter();

        error!("This event will be logged in the root span via tracing::event!");

        // --- Add custom OpenTelemetry Span Events ---
        let current_span = tracing::Span::current();

        // Generate a dynamic request ID using the current time
        let request_id_from_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
            .to_string();

        // Let's assume that this is coming from a request from a user
        let dynamic_attributes = vec![
            KeyValue::new("user.id", "example-user-456"),
            KeyValue::new("request.id", request_id_from_time), // Use timestamp-based ID
        ];

        // Add event with current timestamp using the extension trait
        current_span.add_otel_span_event(
            "Processing dynamic request data",
            dynamic_attributes.clone(),
        );
        println!("Added span event: 'Processing dynamic request data'");

        // Add event with a specific, past timestamp
        let past_timestamp = SystemTime::now() - std::time::Duration::from_secs(1);
        current_span.add_otel_span_event_with_timestamp(
            "Historical data point processed",
            past_timestamp,
            vec![KeyValue::new("historical.record.id", 99)],
        );
        println!("Added span event: 'Historical data point processed' with past timestamp");
    });
}
