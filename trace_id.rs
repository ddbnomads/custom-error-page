use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use log::info;

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_http_context(|_, _| -> Box<dyn HttpContext> { Box::new(HttpHeaders {trace_id: String:: new(), resp_code: String:: new()}) });
}}
  

struct HttpHeaders {
    trace_id: String,
    resp_code: String,
}


impl HttpContext for HttpHeaders {
   fn on_http_request_headers(&mut self, _: usize, _: bool) -> Action {
       // extract x-cloud-trace-context from request headers
       match self.get_http_request_header("x-cloud-trace-context") {
        Some(header_value) => {
            // Find the index of the '/' character.
            let index = header_value.find('/').unwrap();
            // Extract the substring before the '/'.
            let substring = &header_value[..index];
            self.trace_id = substring.to_string(); 
        }
        None => {}
       }
       Action::Continue
       
   }
   fn on_http_response_headers(&mut self, _: usize, _: bool) -> Action {
        match self.get_http_response_header(":status") {
            Some(status) => self.resp_code = status,
            None => {}
        }
        //adds "x-trace-id" response header
        self.add_http_response_header("x-trace-id", &self.trace_id);

        // If there is a Content-Length header and we change the length of
        // the body later, then clients will break. So remove it.
        // We must do this here, because once we exit this function we
        // can no longer modify the response headers.
        self.set_http_response_header("content-length", None);
        Action::Continue
   }
    fn on_http_response_body(&mut self, body_size: usize, _end_of_stream: bool) -> Action {
        if self.resp_code.eq("403") {
            // Append the trace_id into resp_body.
            if let Some(body_bytes) = self.get_http_response_body(0, body_size) {
                let body_str = String::from_utf8(body_bytes).unwrap();
                if body_str.contains("x-trace-id:") {
                    let resp_body_trace_id = &format!("x-trace-id: {}", &self.trace_id);
                    let new_body = body_str.replace("x-trace-id:", &resp_body_trace_id);
                    
                    info!("appened x-trace-id to response body");
                    self.set_http_response_body(0, body_size, &format!("{}", &new_body).into_bytes());
                }
            }
        }
        Action::Continue
    }
}
 
impl Context for HttpHeaders {}