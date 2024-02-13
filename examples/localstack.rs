use aws_config::{meta::region::RegionProviderChain, BehaviorVersion};
use aws_sdk_s3 as s3;
use testcontainers::RunnableImage;
use testcontainers_modules::{localstack::LocalStack, testcontainers::clients::Cli};

#[tokio::main]
async fn main() -> Result<(), s3::Error> {
    pretty_env_logger::init();

    let docker = Cli::default();
    let image: RunnableImage<LocalStack> = LocalStack::default().into();
    let image = image
        .with_env_var(("SERVICES", "s3"))
        .with_mapped_port((34567, 4566));
        
    let container = docker.run(image);
    let host_port = container.get_host_port_ipv4(4566);

    // Set up AWS client
    //let endpoint_url = format!("http://127.0.0.1:{host_port}");
    let endpoint_url = format!("http://host.docker.internal:34567").to_string();
    
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    let creds = s3::config::Credentials::new("fake", "fake", None, None, "test");
    let config = aws_config::defaults(BehaviorVersion::v2023_11_09())
        .region(region_provider)
        .credentials_provider(creds)
        .endpoint_url(endpoint_url)
        .load()
        .await;

    let client = s3::Client::new(&config);

    client
        .create_bucket()
        .bucket("example-bucket")
        .send()
        .await?;

    let list_buckets_output = client.list_buckets().send().await?;
    assert!(list_buckets_output.buckets.is_some());
    let buckets_list = list_buckets_output.buckets.unwrap();
    assert_eq!(1, buckets_list.len());
    assert_eq!("example-bucket", buckets_list[0].name.as_ref().unwrap());

    Ok(())
}
