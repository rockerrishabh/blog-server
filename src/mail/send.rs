use std::env;

use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

pub struct MailOptions {
    pub html_content: String,
    pub to: String,
    pub subject: String,
    pub user: String,
    pub user_name: String,
    pub user_email: String,
}

impl MailOptions {
    pub fn send(self) {
        send_mail(self)
    }
}

pub fn send_mail(options: MailOptions) {
    let smtp_password = env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set");
    let smtp_host = env::var("SMTP_HOST").expect("SMTP_HOST must be set");

    let from = format!("{} <{}>", options.user_name, options.user_email);

    let email = Message::builder()
        .from(from.as_str().parse().unwrap())
        .reply_to(from.as_str().parse().unwrap())
        .to(options.to.as_str().parse().unwrap())
        .subject(options.subject.as_str())
        .header(ContentType::TEXT_HTML)
        .body(options.html_content.to_owned())
        .unwrap();

    let creds = Credentials::new(options.user.to_owned(), smtp_password.to_owned());

    // Open a remote connection to gmail
    let mailer = SmtpTransport::starttls_relay(smtp_host.as_str())
        .unwrap()
        .credentials(creds)
        .build();

    // Send the email
    match mailer.send(&email) {
        Ok(r) => println!("Email sent: {:?}", r),
        Err(e) => panic!("Could not send email: {e:?}"),
    }
}
