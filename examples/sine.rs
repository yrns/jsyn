use janetrs::client::JanetClient;

fn main() {
    let client = JanetClient::init_with_default_env().unwrap();
    let _ = client
        .run(std::fs::read_to_string("examples/sine.janet").unwrap())
        .unwrap();
}
