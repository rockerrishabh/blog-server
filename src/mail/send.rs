use std::env;
extern crate imap;
extern crate native_tls;

use imap::types::Flag;
use lettre::message::header::ContentType;

use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use native_tls::TlsConnector;

pub struct MailOptions {
    pub html_content: String,
    pub to: String,
    pub subject: String,
    pub user_name: String,
    pub user_email: String,
}

impl MailOptions {
    pub fn send(self) {
        send_mail(self)
    }
}

fn save_to_sent_folder(message: &Message) -> Result<(), Box<dyn std::error::Error>> {
    let smtp_host = env::var("SMTP_HOST").expect("SMTP_HOST must be set");
    let smtp_password = env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set");

    let tls = TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    let client = imap::connect((smtp_host.as_str(), 993), smtp_host.as_str(), &tls)?;

    let from_email = message
        .envelope()
        .from()
        .ok_or("Could not extract from email")?;

    let mut imap_session = client.login(&from_email, &smtp_password).map_err(|e| e.0)?;

    let raw_message = message.formatted();

    let mailboxes = imap_session.list(None, Some("*"))?;
    let sent_folder = mailboxes
        .iter()
        .find(|box_| box_.name().to_ascii_lowercase().contains("sent"))
        .map(|box_| box_.name())
        .unwrap_or("Sent");

    if let Err(_) = imap_session.select(sent_folder) {
        imap_session.create(sent_folder)?;
    }

    let flags = vec![Flag::Recent];

    imap_session.append_with_flags(sent_folder, &raw_message, &flags)?;

    imap_session.logout()?;

    Ok(())
}

pub fn send_mail(options: MailOptions) {
    let smtp_password = env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set");
    let smtp_host = env::var("SMTP_HOST").expect("SMTP_HOST must be set");

    let from = format!("{} <{}>", options.user_name, &options.user_email);

    let message = Message::builder()
        .from(from.as_str().parse().unwrap())
        .reply_to(from.as_str().parse().unwrap())
        .to(options.to.as_str().parse().unwrap())
        .subject(options.subject.as_str())
        .header(ContentType::TEXT_HTML)
        .body(options.html_content.to_owned())
        .unwrap();

    let creds = Credentials::new(options.user_email.to_owned(), smtp_password.to_owned());

    let mailer = SmtpTransport::starttls_relay(smtp_host.as_str())
        .unwrap()
        .credentials(creds)
        .build();

    match mailer.send(&message) {
        Ok(_) => {
            println!("Email sent successfully!");
            if let Err(e) = save_to_sent_folder(&message) {
                println!("Failed to save to Sent folder: {:?}", e);
            }
        }
        Err(e) => {
            println!("Failed to send email: {e:?}");
        }
    }
}
