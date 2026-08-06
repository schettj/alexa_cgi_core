# alexa_cgi_core

A lightweight, zero-dependency, ultra-fast CGI framework for self-hosted Amazon Alexa skills in Rust.

[![Crates.io](https://shields.io)](https://crates.io)
[![License: MIT](https://shields.io)](https://opensource.org)

Unlike heavy asynchronous web servers (Actix, Axum, Rocket) or complex cloud serverless infrastructure (AWS Lambda), `alexa_cgi_core` utilizes the classic, proven Common Gateway Interface (CGI) pipeline. It orchestrates Alexa requests directly through standard input (`stdin`) and standard output (`stdout`), yielding minimal binary sizes, zero long-running daemon memory overhead, and instant cold-start execution times.

---

## 🔥 Features

* **Zero-Downtime Architecture**: Served as a short-lived transient binary process managed natively by your local web server (Apache, Nginx, Hiawatha).
* **Automated Security Verification**:
  * **Timestamp Validation**: Automatically enforces Amazon's strict 150-second cryptographic replay attack window boundary check.
  * **Application ID Enforcement**: Drops unauthorized third-party endpoint scraping traffic before executing business logic.
* **Fluent Response Builder**: Features a clean, programmatic builder pattern for assembling compliant Alexa PlainText speech frames.
* **Declarative Trait Blueprint**: Abstract away all `stdin`/`stdout` serialization boilerplate behind a single cleanly mapped Rust trait.

---

## 🛠️ Installation

Add `alexa_cgi_core` to your project's `Cargo.toml` dependencies:

```toml
[dependencies]
alexa_cgi_core = "0.1.0"
```

For custom home-automation or private server setups, you can link it directly via local path configurations:

```toml
[dependencies]
alexa_cgi_core = { path = "../alexa_cgi_core" }
```

To minimize production execution footprint, optimize your release profile:

```toml
[profile.release]
opt-level = "z"      # Optimize strictly for minimal binary footprint size
lto = true           # Enable Link-Time Optimization
codegen-units = 1    # Maximize optimization passes
panic = "abort"      # Strip diagnostic stack unwinding structures
```

---

## 💻 Usage Example

### 1. Define the Skill Handler (`src/alexa.rs`)

Implement the `AlexaSkill` trait on your custom struct. The trait routes core lifecycle behaviors seamlessly:

```rust
use alexa_cgi_core::AlexaSkill;

pub struct HomeAssistantSkill;

impl AlexaSkill for HomeAssistantSkill {
    // Return your explicit Amazon Skill ID to enforce security validation gates
    fn skill_id(&self) -> Option<&str> {
        Some("amzn1.echo-api.skill.your-actual-skill-id-here")
    }

    fn handle_launch(&self) -> String {
        String::from("Welcome to your self-hosted assistant. You can ask for status.")
    }

    fn handle_intent(&self, intent_name: &str) -> String {
        match intent_name {
            "StatusIntent" => String::from("All systems are completely balanced and standing by."),
            _ => String::from("I didn't quite catch that. Please try again.")
        }
    }

    fn handle_fallback(&self) -> String {
        String::from("System fallback triggered. Try asking for system status.")
    }

    fn handle_session_ended(&self) {
        // Optional cleanup logic executes here on session teardown close
    }
}
```

### 2. Configure the Entrypoint (`src/main.rs`)

Pass your handler instance straight into the core generic execution engine driver:

```rust
mod alexa;
use alexa::HomeAssistantSkill;

fn main() {
    let skill = HomeAssistantSkill;
    
    // Executes the global environment checks, request validation, and stream pipeline
    alexa_cgi_core::run_cgi_skill(skill);
}
```

---

## 🌐 Server Deployment (Apache 2.4 Example)

### 1. Deploy the Compiled Binary
Compile your binary and copy it directly over into your server's executable CGI directory:

```bash
cargo build --release
sudo cp target/release/your_skill_binary /usr/lib/cgi-bin/skill.cgi
sudo chmod +x /usr/lib/cgi-bin/skill.cgi
```

### 2. Configure Apache SSL VirtualHost
Amazon **requires** all Alexa endpoints to be served over secure HTTPS with a valid SSL/TLS certificate. Map your legacy or preferred script path alias inside your active port `443` configuration layout file (e.g., `/etc/apache2/sites-enabled/000-default-le-ssl.conf`):

```apache
<VirtualHost *:443>
    ServerName yourdomain.com

    # Map the clean endpoint path straight to your high-speed compiled Rust CGI binary
    Alias /skill.php /usr/lib/cgi-bin/skill.cgi

    <Directory "/usr/lib/cgi-bin">
        AllowOverride None
        Options +ExecCGI
        AddHandler cgi-script .cgi
        Require all granted
    </Directory>
</VirtualHost>
```

Activate `mod_cgi` and restart the server engine:

```bash
sudo a2enmod cgi
sudo apache2ctl configtest
sudo systemctl restart apache2
```

---

## 🔒 Security Requirements Note

This framework natively satisfies Amazon's runtime production constraints by implementing strict **Application ID filtering** and **Timestamp delta validation**. 

*Note: For official public Amazon Skill Store certification, inbound requests must also pass a raw cryptographic certificate chain signature check. If you plan to publish your skill publicly to the global marketplace, ensure your server network architecture terminates signature verification at a proxy layer or verify certificates via an additional validation crate.*

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
