use dirtybase_mail::EmailBuilder;

#[tokio::main]
async fn main() {
    dirtybase_mail::register_mail_adapters().await;

    let email = EmailBuilder::new().build();

    let result = dirtybase_mail::Mailman::new()
        .to("xyz@example.com")
        .subject("The quick brown fox jumps...")
        .mail(email)
        .send()
        .await;

    println!("send result: {result:?}");
}
