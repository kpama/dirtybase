use lettre::{
    message::header::ContentType,
    transport::smtp::{authentication::Credentials, client::Tls},
    Message, SmtpTransport, Transport,
};

use crate::{email::Envelope, AdapterTrait};

pub struct SmtpAdapter;

#[async_trait::async_trait]
impl AdapterTrait for SmtpAdapter {
    fn name(&self) -> &str {
        "smtp"
    }

    async fn send(&self, envelope: Envelope) -> Result<bool, anyhow::Error> {
        let email = Message::builder()
            .from("NoBody <nobody@domain.tld>".parse().unwrap())
            .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
            .to("Hei <hei@domain.tld>".parse().unwrap())
            .subject("Happy new year")
            .header(ContentType::TEXT_PLAIN)
            .body(envelope.subject.unwrap_or_default())
            .unwrap();

        let _creds = Credentials::new("smtp_username".to_owned(), "smtp_password".to_owned());

        // Open a remote connection to gmail
        let mailer = SmtpTransport::relay("mail")
            .unwrap()
            .port(1025)
            .tls(Tls::None)
            // .credentials(creds)
            .build();

        // Send the email
        match mailer.send(&email) {
            Ok(_) => println!("Email sent successfully!"),
            Err(e) => panic!("Could not send email: {:?}", e),
        }
        Ok(true)
    }
}
