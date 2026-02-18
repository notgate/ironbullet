import type { BlockDoc } from './types';

export const REQUEST_DOCS: BlockDoc[] = [
	{
		type: 'HttpRequest',
		name: 'HTTP Request',
		category: 'Requests',
		description: 'Sends an HTTP request to a URL and stores the response in variables. Supports GET, POST, PUT, DELETE, PATCH with custom headers, body, and cookies.',
		parameters: [
			{ name: 'method', type: 'string', required: true, description: 'HTTP method (GET, POST, PUT, DELETE, PATCH, HEAD)', default: 'GET' },
			{ name: 'url', type: 'string', required: true, description: 'Target URL. Supports variable interpolation with <VAR>' },
			{ name: 'headers', type: 'array', required: false, description: 'Custom HTTP headers as key-value pairs' },
			{ name: 'body', type: 'string', required: false, description: 'Request body content' },
			{ name: 'body_type', type: 'enum', required: false, description: 'Body encoding: None, Standard, Raw, Multipart, BasicAuth', default: 'None' },
			{ name: 'content_type', type: 'string', required: false, description: 'Content-Type header value', default: 'application/x-www-form-urlencoded' },
			{ name: 'follow_redirects', type: 'boolean', required: false, description: 'Follow HTTP redirects automatically', default: 'true' },
			{ name: 'timeout_ms', type: 'number', required: false, description: 'Request timeout in milliseconds', default: '10000' },
			{ name: 'response_var', type: 'string', required: false, description: 'Variable prefix for response storage', default: 'SOURCE' },
		],
		codeExample: `URL: https://example.com/api/login
Method: POST
Body: username=<USER>&password=<PASS>
→ Stores response in data.SOURCE, status in data.RESPONSECODE`,
		tips: [
			'Response body is stored in data.{response_var}, headers in data.{response_var}.HEADERS',
			'Status code is always available as data.RESPONSECODE',
			'Use <USER> and <PASS> to inject credentials from the wordlist',
			'Custom cookies are sent in addition to the cookie jar',
		],
		relatedBlocks: ['ParseLR', 'ParseRegex', 'ParseJSON', 'KeyCheck'],
		rustCode: `// Variable interpolation on all string fields
let url = self.variables.interpolate(&settings.url);
let body = self.variables.interpolate(&settings.body);
let headers: Vec<(String, String)> = settings.headers.iter()
    .map(|(k, v)| (self.variables.interpolate(k), self.variables.interpolate(v)))
    .collect();

// Send via sidecar (TLS fingerprint-aware HTTP client)
let req = SidecarRequest {
    method: settings.method.clone(),
    url: url.clone(),
    headers, body,
    proxy: self.proxy.clone(),
    ja3: self.override_ja3.clone(),
    follow_redirects: settings.follow_redirects,
    timeout: settings.timeout_ms,
};
let resp = sidecar_tx.send(req).await?;

// Store response in variable namespace
let pfx = &settings.response_var; // default: "SOURCE"
self.variables.set_data(pfx, resp.body);
self.variables.set_data(&format!("{pfx}.STATUS"), resp.status.to_string());
self.variables.set_data(&format!("{pfx}.HEADERS"), serde_json::to_string(&resp.headers)?);
self.variables.set_data(&format!("{pfx}.COOKIES"), serde_json::to_string(&resp.cookies)?);
self.variables.set_data("RESPONSECODE", resp.status.to_string());`,
	},
	{
		type: 'TcpRequest',
		name: 'TCP Request',
		category: 'Requests',
		description: 'Sends raw data over a TCP connection and reads the response. Supports TLS for encrypted connections.',
		parameters: [
			{ name: 'host', type: 'string', required: true, description: 'Target hostname or IP' },
			{ name: 'port', type: 'number', required: true, description: 'Target port number', default: '80' },
			{ name: 'data', type: 'string', required: false, description: 'Raw data to send' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store response', default: 'TCP_RESPONSE' },
			{ name: 'use_tls', type: 'boolean', required: false, description: 'Use TLS encryption', default: 'false' },
			{ name: 'timeout_ms', type: 'number', required: false, description: 'Connection timeout', default: '5000' },
		],
		codeExample: `Host: smtp.example.com
Port: 25
Data: EHLO example.com\\r\\n`,
		tips: [
			'Use \\r\\n for line endings in text protocols',
			'Enable TLS for secure connections (port 443, 993, etc.)',
		],
		relatedBlocks: ['UdpRequest', 'HttpRequest', 'SshRequest'],
		rustCode: `let host = self.variables.interpolate(&settings.host);
let addr = format!("{}:{}", host, settings.port);
let mut stream = TcpStream::connect(&addr).await?;
let data = self.variables.interpolate(&settings.data);
stream.write_all(data.as_bytes()).await?;
let mut buf = vec![0u8; 8192];
let n = tokio::time::timeout(
    Duration::from_millis(settings.timeout_ms),
    stream.read(&mut buf),
).await??;
let response = String::from_utf8_lossy(&buf[..n]).to_string();
self.variables.set_user(&settings.output_var, response, settings.capture);`,
	},
	{
		type: 'UdpRequest',
		name: 'UDP Request',
		category: 'Requests',
		description: 'Sends a UDP datagram and optionally reads a response. Useful for DNS queries and other UDP protocols.',
		parameters: [
			{ name: 'host', type: 'string', required: true, description: 'Target hostname or IP' },
			{ name: 'port', type: 'number', required: true, description: 'Target port', default: '53' },
			{ name: 'data', type: 'string', required: false, description: 'Raw data to send' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store response', default: 'UDP_RESPONSE' },
			{ name: 'timeout_ms', type: 'number', required: false, description: 'Timeout in ms', default: '5000' },
		],
		codeExample: `Host: 8.8.8.8, Port: 53
→ Send DNS query and read response`,
		tips: ['UDP is connectionless — no guarantee of delivery', 'Useful for DNS, SNMP, and other UDP-based protocols'],
		relatedBlocks: ['TcpRequest', 'HttpRequest'],
		rustCode: `let host = self.variables.interpolate(&settings.host);
let addr = format!("{}:{}", host, settings.port);
let socket = UdpSocket::bind("0.0.0.0:0").await?;
let data = self.variables.interpolate(&settings.data);
socket.send_to(data.as_bytes(), &addr).await?;
let mut buf = vec![0u8; 8192];
let n = tokio::time::timeout(
    Duration::from_millis(settings.timeout_ms),
    socket.recv(&mut buf),
).await??;
let response = String::from_utf8_lossy(&buf[..n]).to_string();
self.variables.set_user(&settings.output_var, response, settings.capture);`,
	},
	{
		type: 'FtpRequest',
		name: 'FTP Request',
		category: 'Requests',
		description: 'Connects to an FTP server and executes a command. Supports login with username/password.',
		parameters: [
			{ name: 'host', type: 'string', required: true, description: 'FTP server hostname' },
			{ name: 'port', type: 'number', required: true, description: 'FTP port', default: '21' },
			{ name: 'username', type: 'string', required: true, description: 'FTP username' },
			{ name: 'password', type: 'string', required: true, description: 'FTP password' },
			{ name: 'command', type: 'string', required: false, description: 'FTP command to execute', default: 'LIST' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store response', default: 'FTP_RESPONSE' },
		],
		codeExample: `Host: ftp.example.com
Username: <USER>, Password: <PASS>
Command: LIST
→ Lists directory contents after login`,
		tips: ['Common commands: LIST, STAT, PWD, SIZE filename'],
		relatedBlocks: ['SshRequest', 'TcpRequest'],
		rustCode: `let host = self.variables.interpolate(&settings.host);
let addr = format!("{}:{}", host, settings.port);
let mut stream = TcpStream::connect(&addr).await?;
let _ = read_response(&mut stream).await?; // 220 welcome
send_cmd(&mut stream, &format!("USER {}\\r\\n", settings.username)).await?;
send_cmd(&mut stream, &format!("PASS {}\\r\\n", settings.password)).await?;
let command = self.variables.interpolate(&settings.command);
let response = send_cmd(&mut stream, &format!("{}\\r\\n", command)).await?;
send_cmd(&mut stream, "QUIT\\r\\n").await?;
self.variables.set_user(&settings.output_var, response, settings.capture);`,
	},
	{
		type: 'SshRequest',
		name: 'SSH Request',
		category: 'Requests',
		description: 'Connects to an SSH server and executes a remote command.',
		parameters: [
			{ name: 'host', type: 'string', required: true, description: 'SSH server hostname' },
			{ name: 'port', type: 'number', required: true, description: 'SSH port', default: '22' },
			{ name: 'username', type: 'string', required: true, description: 'SSH username' },
			{ name: 'password', type: 'string', required: true, description: 'SSH password' },
			{ name: 'command', type: 'string', required: true, description: 'Remote command to execute' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store output', default: 'SSH_RESPONSE' },
		],
		codeExample: `Host: server.example.com
Username: <USER>, Password: <PASS>
Command: whoami
→ Executes "whoami" and stores output`,
		tips: ['Connection will timeout if credentials are wrong', 'Output includes both stdout and stderr'],
		relatedBlocks: ['TcpRequest', 'FtpRequest'],
		rustCode: `let host = self.variables.interpolate(&settings.host);
let addr = format!("{}:{}", host, settings.port);
let mut stream = TcpStream::connect(&addr).await?;
// SSH transport negotiation + password auth
let command = self.variables.interpolate(&settings.command);
// Execute command and capture stdout
let response = ssh_exec(&mut stream, &settings.username, &settings.password, &command).await?;
self.variables.set_user(&settings.output_var, response, settings.capture);`,
	},
	{
		type: 'ImapRequest',
		name: 'IMAP Request',
		category: 'Requests',
		description: 'Connects to an IMAP mail server to check or fetch email.',
		parameters: [
			{ name: 'host', type: 'string', required: true, description: 'IMAP server hostname' },
			{ name: 'port', type: 'number', required: true, description: 'IMAP port', default: '993' },
			{ name: 'username', type: 'string', required: true, description: 'Email username' },
			{ name: 'password', type: 'string', required: true, description: 'Email password' },
			{ name: 'use_tls', type: 'boolean', required: false, description: 'Use TLS encryption', default: 'true' },
			{ name: 'command', type: 'string', required: false, description: 'IMAP command', default: 'LOGIN' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store response', default: 'IMAP_RESPONSE' },
		],
		codeExample: `Host: imap.gmail.com:993
Username: <USER>, Password: <PASS>
Command: LOGIN
→ Attempts IMAP login to verify credentials`,
		tips: ['Port 993 = IMAPS (TLS), Port 143 = IMAP (plain)'],
		relatedBlocks: ['SmtpRequest', 'PopRequest'],
		rustCode: `let host = self.variables.interpolate(&settings.host);
let addr = format!("{}:{}", host, settings.port);
let mut stream = TcpStream::connect(&addr).await?;
let _ = read_line(&mut stream).await?; // * OK banner
send_cmd(&mut stream, &format!("A1 LOGIN {} {}\\r\\n", settings.username, settings.password)).await?;
let command = self.variables.interpolate(&settings.command);
let response = send_cmd(&mut stream, &format!("A2 {}\\r\\n", command)).await?;
send_cmd(&mut stream, "A3 LOGOUT\\r\\n").await?;
self.variables.set_user(&settings.output_var, response, settings.capture);`,
	},
	{
		type: 'SmtpRequest',
		name: 'SMTP Request',
		category: 'Requests',
		description: 'Connects to an SMTP server to send email or verify credentials.',
		parameters: [
			{ name: 'host', type: 'string', required: true, description: 'SMTP server hostname' },
			{ name: 'port', type: 'number', required: true, description: 'SMTP port', default: '587' },
			{ name: 'username', type: 'string', required: true, description: 'SMTP username' },
			{ name: 'password', type: 'string', required: true, description: 'SMTP password' },
			{ name: 'use_tls', type: 'boolean', required: false, description: 'Use TLS', default: 'true' },
			{ name: 'command', type: 'string', required: false, description: 'SMTP command', default: 'EHLO' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store response', default: 'SMTP_RESPONSE' },
		],
		codeExample: `Host: smtp.gmail.com:587
Command: EHLO
→ Initiates SMTP handshake`,
		tips: ['Port 587 = submission (STARTTLS), Port 465 = SMTPS, Port 25 = plain'],
		relatedBlocks: ['ImapRequest', 'PopRequest'],
		rustCode: `let host = self.variables.interpolate(&settings.host);
let addr = format!("{}:{}", host, settings.port);
let mut stream = TcpStream::connect(&addr).await?;
let _ = read_line(&mut stream).await?; // 220 banner
send_cmd(&mut stream, "EHLO ironbullet\\r\\n").await?;
// AUTH LOGIN if credentials provided
if !settings.username.is_empty() {
    send_cmd(&mut stream, "AUTH LOGIN\\r\\n").await?;
    send_cmd(&mut stream, &format!("{}\\r\\n", base64_encode(&settings.username))).await?;
    send_cmd(&mut stream, &format!("{}\\r\\n", base64_encode(&settings.password))).await?;
}
let command = self.variables.interpolate(&settings.command);
let response = send_cmd(&mut stream, &format!("{}\\r\\n", command)).await?;
self.variables.set_user(&settings.output_var, response, settings.capture);`,
	},
	{
		type: 'PopRequest',
		name: 'POP Request',
		category: 'Requests',
		description: 'Connects to a POP3 mail server to retrieve email or verify credentials.',
		parameters: [
			{ name: 'host', type: 'string', required: true, description: 'POP3 server hostname' },
			{ name: 'port', type: 'number', required: true, description: 'POP3 port', default: '995' },
			{ name: 'username', type: 'string', required: true, description: 'Email username' },
			{ name: 'password', type: 'string', required: true, description: 'Email password' },
			{ name: 'use_tls', type: 'boolean', required: false, description: 'Use TLS', default: 'true' },
			{ name: 'command', type: 'string', required: false, description: 'POP3 command', default: 'STAT' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store response', default: 'POP_RESPONSE' },
		],
		codeExample: `Host: pop.gmail.com:995
Command: STAT
→ Returns mailbox statistics (message count, size)`,
		tips: ['STAT returns message count, LIST shows individual messages'],
		relatedBlocks: ['ImapRequest', 'SmtpRequest'],
		rustCode: `let host = self.variables.interpolate(&settings.host);
let addr = format!("{}:{}", host, settings.port);
let mut stream = TcpStream::connect(&addr).await?;
let _ = read_line(&mut stream).await?; // +OK banner
send_cmd(&mut stream, &format!("USER {}\\r\\n", settings.username)).await?;
send_cmd(&mut stream, &format!("PASS {}\\r\\n", settings.password)).await?;
let command = self.variables.interpolate(&settings.command);
let response = send_cmd(&mut stream, &format!("{}\\r\\n", command)).await?;
send_cmd(&mut stream, "QUIT\\r\\n").await?;
self.variables.set_user(&settings.output_var, response, settings.capture);`,
	},
];
