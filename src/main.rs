use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let configuration = get_configuration().expect("Failed to read configuration.");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    // Create a TCP listener on the specified address
    let listener = std::net::TcpListener::bind(&address)
        .expect("Failed to bind to address");
    run(listener)?.await
}