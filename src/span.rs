use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use tracing::Span;
use tracing_actix_web::{DefaultRootSpanBuilder, RootSpanBuilder};

pub(crate) struct DomainRootSpanBuilder;

impl RootSpanBuilder for DomainRootSpanBuilder {
    fn on_request_start(request: &ServiceRequest) -> Span {
        let root_span = tracing_actix_web::root_span!(request);
        root_span
    }
    fn on_request_end<B: MessageBody>(
        span: Span,
        outcome: &Result<ServiceResponse<B>, actix_web::Error>,
    ) {
        DefaultRootSpanBuilder::on_request_end(span, outcome);
    }
}
