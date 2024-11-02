use std::env;

pub fn verification_template(
    name: String,
    email: String,
    token: String,
) -> (String, String, String, String, String, String) {
    let subject = "Verify Your Email".to_string();
    let to = format!("{} <{}>", &name, &email);
    let smtp_verification_name = "Verification Team".to_string();
    let smtp_verification_user =
        env::var("SMTP_VERIFICATION_USER").expect("SMTP_VERIFICATION_USER must be set");
    let smtp_verification_email =
        env::var("SMTP_VERIFICATION_EMAIL").expect("SMTP_VERIFICATION_EMAIL must be set");
    let html = format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <meta charset="UTF-8">
            <title>Verify Your Email</title>
            <style>
                body {{
                    font-family: Arial, sans-serif;
                    background-color: #f5f5f5;
                }}

                .container {{
                    max-width: 600px;
                    margin: 0 auto;
                    padding: 20px;
                    background-color: #fff;
                    border-radius: 5px;
                    box-shadow: 0 2px 5px rgba(0, 0, 0, 0.1);
                }}

                h1 {{
                    color: #333;
                    text-align: center;
                    margin-bottom: 20px;
                }}

                p {{
                    color: #666;
                    line-height: 1.5;
                    margin-bottom: 10px;
                }}

                a {{
                    color: #007bff;
                    text-decoration: none;
                    font-weight: bold;
                }}
            </style>
        </head>
        <body>
            <div class="container">
                <h1>Hello, {}!</h1>
                <p>Thank you for registering with us. Please verify your email address by clicking the link below:</p>
                <p><a href="http://localhost:3000/verify?token={}">Verify Your Email</a></p>
                <p>If you didn't request this verification, please ignore this email.</p>
            </div>
        </body>
        </html>
        "#,
        &name, &token
    );

    return (
        subject,
        to,
        html,
        smtp_verification_name,
        smtp_verification_user,
        smtp_verification_email,
    );
}
