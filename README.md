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

Or reference your hosted GitHub repository securely over SSH:

```toml
[dependencies]
alexa_cgi_core = { git = "git@github.com:yourusername/alexa_cgi_core.git", branch = "main" }
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
sudo cp target/release/your_skill_binary /usr/lib/cgi-bin/service1.cgi
sudo chmod +x /usr/lib/cgi-bin/service1.cgi
```

### 2. Configure Apache SSL VirtualHost
Amazon **requires** all Alexa endpoints to be served over secure HTTPS with a valid SSL/TLS certificate. Map your script endpoint directly inside your active port `443` configuration layout file (e.g., `/etc/apache2/sites-enabled/000-default-le-ssl.conf`):

```apache
<VirtualHost *:443>
    ServerName yourdomain.com

    # Standard CGI runtime mapping definition routing web hits to your executable
    ScriptAlias /skill1 /usr/lib/cgi-bin/service1.cgi

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

## 🔒 Offloading Amazon Cryptographic Verification in Apache (Generic Proxy Method)

For official public Amazon Skill Store certification, inbound requests must pass an intensive cryptographic certificate chain signature check. To keep your Rust CGI binary lightweight and lightning-fast, you can offload this verification step to an Apache proxy authentication layer using `mod_wsgi` and the Python `ask-sdk-webservice-support` package.

### 1. Install Server Dependencies
```bash
sudo apt-get install libapache2-mod-wsgi-py3 python3-pip
sudo pip3 install ask-sdk-webservice-support requests
sudo a2enmod wsgi proxy proxy_http
```

### 2. Create the Generic Verification Wrapper (`/usr/lib/cgi-bin/alexa_verify.py`)
This generic WSGI script reads the target executable pathway dynamically out of Apache environment variables, allowing a single script instance to power an infinite array of underlying skills seamlessly:

```python
import os
import subprocess
from ask_sdk_webservice_support.verifier import RequestVerifier, VerificationException

def application(environ, start_response):
    try:
        # 1. Look up the generic target binary path injected by Apache environment rules
        target_binary = environ.get('TARGET_CGI_BIN', '')
        if not target_binary or not os.path.exists(target_binary):
            start_response('500 Internal Server Error', [('Content-Type', 'text/plain')])
            return [f"Target executable configuration mismatch: '{target_binary}' not found.".encode('utf-8')]

        # 2. Gather Amazon's mandatory security headers
        signature_url = environ.get('HTTP_SIGNATURECERTCHAINURL', '')
        signature = environ.get('HTTP_SIGNATURE', '')
        
        content_length = int(environ.get('CONTENT_LENGTH', 0))
        request_body = environ['wsgi.input'].read(content_length)

        # 3. Execute strict cryptographic validation check via official Amazon algorithms
        verifier = RequestVerifier()
        verifier.verify(request_body.decode('utf-8'), signature, signature_url)

        # 4. Signature is valid! Forward body payload stream to your target generic binary
        proc = subprocess.Popen(
            [target_binary],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
            env=os.environ.copy()
        )
        stdout_data, _ = proc.communicate(input=request_body)

        # 5. Return the compiled response payload transparently
        start_response('200 OK', [('Content-Type', 'application/json;charset=UTF-8')])
        return [stdout_data]

    except VerificationException as e:
        start_response('400 Bad Request', [('Content-Type', 'text/plain')])
        return [b"Cryptographic validation failure."]
    except Exception as e:
        start_response('500 Internal Server Error', [('Content-Type', 'text/plain')])
        return [str(e).encode('utf-8')]
```

### 3. Update Apache SSL VirtualHost Mapping (Deploying Multiple Skills Generically)
Use Apache's `WSGIScriptAlias` combined with `<Location>` blocks to establish endpoints for `service1.cgi` and `service2.cgi`, passing the target paths via `SetEnv TARGET_CGI_BIN`:

```apache
<VirtualHost *:443>
    ServerName yourdomain.com

    # -------------------------------------------------------------
    # Skill A Mapping (Service 1 Gateway)
    # -------------------------------------------------------------
    WSGIScriptAlias /skill1 /usr/lib/cgi-bin/alexa_verify.py
    <Location /skill1>
        SetEnv TARGET_CGI_BIN /usr/lib/cgi-bin/service1.cgi
    </Location>

    # -------------------------------------------------------------
    # Skill B Mapping (Service 2 Gateway)
    # -------------------------------------------------------------
    WSGIScriptAlias /skill2 /usr/lib/cgi-bin/alexa_verify.py
    <Location /skill2>
        SetEnv TARGET_CGI_BIN /usr/lib/cgi-bin/service2.cgi
    </Location>

    <Directory "/usr/lib/cgi-bin">
        Options +ExecCGI
        Require all granted
    </Directory>
</VirtualHost>
```

Restart Apache (`sudo systemctl restart apache2`) to apply the configuration change.

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
